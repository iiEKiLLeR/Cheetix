"""Builds data/ops.json: tweaks described as operations the app's engine applies with a
saved backup of the original state. Run: python tools/gen_ops.py (checks for duplicates).

Each tweak changes exactly one setting (or the values that together make one setting).
The src argument is a maintainer note only and is not written to the data.
"""
import json, os, re, sys

POL = r"HKLM\SOFTWARE\Policies\Microsoft"
CDM = r"HKCU\Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager"
ADV = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced"
FS = r"HKLM\SYSTEM\CurrentControlSet\Control\FileSystem"
MM_KEY = r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management"
DNSPOL = r"HKLM\SOFTWARE\Policies\Microsoft\Windows NT\DNSClient"
DNSCACHE = r"HKLM\SYSTEM\CurrentControlSet\Services\Dnscache\Parameters"
EDGE = r"HKLM\SOFTWARE\Policies\Microsoft\Edge"
NIC = r"HKLM\SYSTEM\CurrentControlSet\Control\Class\{4d36e972-e325-11ce-bfc1-08002be10318}"
AUTOLOG = r"HKLM\SYSTEM\CurrentControlSet\Control\WMI\Autologger"
PROC = "54533251-82be-4824-96c1-47b60b740d00"
EDGE_NOTE = "Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off."


# ---- operations -------------------------------------------------------------
def R(p, n, v, k="dword", d=None, gate=None, must=False):
    """Registry value. v=None means 'value must not exist'. d = Windows default (None = absent),
    used only when no backup was saved. gate = key that must exist or the op is skipped."""
    op = {"t": "reg", "p": p, "n": n, "k": k, "v": v, "d": d}
    if gate or must:
        op["gate"] = gate or p
    return op


def S(name, d, start=4):
    """Service start type (written to its Start value). Skipped if the service doesn't exist."""
    return {"t": "svc", "n": name, "s": start, "d": d}


def TK(full):
    i = full.rfind("\\")
    return {"t": "task", "p": full[: i + 1], "n": full[i + 1 :], "d": True}


def PW(sub, setting, ac, dc=None, d=None):
    """Power-plan setting of the active plan (AC, optionally DC)."""
    return {"t": "pwr", "sub": sub, "set": setting, "ac": ac, "dc": dc, "d": d}


def KEYS(base, n, v, k="sz", has=None, only_existing=True, sub=r"^\d{4}$"):
    """Same value on every subkey of base (e.g. each network adapter) that matches."""
    return {"t": "keys", "base": base, "sub": sub, "has": has, "n": n, "k": k, "v": v, "only": only_existing}


def MMA(name, d=True):
    return {"t": "mm", "n": name, "on": False, "d": d}


def FE(name):
    return {"t": "feat", "n": name, "d": None}


TWEAKS = []


def T(c, id, name, desc, ops, why, gain, cost, who, restart, risk, src, builds=None, needs=None, note=None):
    assert restart in ("none", "explorer", "signout", "reboot", "app", "adapter"), id
    assert risk in ("low", "medium", "high"), id
    t = {"c": c, "id": id, "name": name, "desc": desc, "ops": ops,
         "info": {"why": why, "gain": gain, "cost": cost, "who": who, "restart": restart, "risk": risk}}
    if note:
        t["info"]["note"] = note
    if builds:
        t["builds"] = builds
    if needs:
        t["needs"] = needs
    TWEAKS.append(t)


OC = "documented setting"
EX = "documented Windows setting"
W11 = [22000, 99999]
W10 = [0, 21999]

# ============================ PRIVACY & TELEMETRY ============================
T("privacy", "activity_history_policy", "DISABLE ACTIVITY HISTORY (POLICY)",
  "Group-policy switch that stops Windows from recording which apps, files and websites you open (Activity History) and from uploading that history to your Microsoft account.",
  [R(POL + r"\Windows\System", "EnableActivityFeed", 0), R(POL + r"\Windows\System", "PublishUserActivities", 0), R(POL + r"\Windows\System", "UploadUserActivities", 0)],
  "Keeps a local and cloud timeline of your activity from being built.",
  "Less personal data stored and synced; removes a small amount of background logging.",
  "'Pick up where you left off' suggestions and cross-device activity resume stop working.",
  "Everyone who doesn't use cross-device activity resume.", "signout", "low", OC)
T("privacy", "location_all_users", "DISABLE LOCATION FOR ALL USERS",
  "Turns off the device-wide location switch (the top switch in Settings > Privacy > Location), so no account or app on this PC can read your position. The existing Location tweak only affects your own account's apps.",
  [R(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\location", "Value", "Deny", "sz", d="Allow"),
   R(r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Sensor\Overrides\{BFA794E4-F964-4FDB-90F6-51056BFE4B44}", "SensorPermissionState", 0, d=1)],
  "Stops every user and app from obtaining your physical location.",
  "No location lookups (Wi-Fi/IP based or GPS) by any app.",
  "Maps, weather, 'Find my device', automatic time zone and any app that needs your location stop getting it.",
  "Desktop PCs; anyone who never uses location features.", "none", "low", OC)
T("privacy", "maps_auto_update_off", "DISABLE OFFLINE MAPS AUTO-UPDATE",
  "Stops Windows from automatically downloading updates for offline maps (Settings > Apps > Offline maps).",
  [R(r"HKLM\SYSTEM\Maps", "AutoUpdateEnabled", 0, d=1)],
  "Avoids background map downloads you may never use.",
  "No automatic map downloads over your connection.",
  "Offline maps you downloaded go out of date until you update them by hand.",
  "Everyone not using the Maps app offline.", "none", "low", OC)
T("privacy", "feedback_notifications_off", "BLOCK FEEDBACK NOTIFICATIONS (POLICY)",
  "Policy that stops Windows and the Feedback Hub from showing 'How likely are you to recommend Windows…' survey pop-ups for every user.",
  [R(POL + r"\Windows\DataCollection", "DoNotShowFeedbackNotifications", 1)],
  "Removes unsolicited survey pop-ups.", "No survey interruptions.",
  "None; you can still open Feedback Hub yourself.", "Everyone.", "none", "low", OC)
T("privacy", "feedback_frequency_never", "FEEDBACK FREQUENCY: NEVER",
  "Sets your account's 'Feedback frequency' (Settings > Privacy > Diagnostics & feedback) to Never by setting the survey count to 0 and removing the survey period.",
  [R(r"HKCU\SOFTWARE\Microsoft\Siuf\Rules", "NumberOfSIUFInPeriod", 0), R(r"HKCU\SOFTWARE\Microsoft\Siuf\Rules", "PeriodInNanoSeconds", None, "qword")],
  "The per-account counterpart of the feedback policy above.", "No feedback requests for your account.",
  "None.", "Everyone.", "none", "low", OC)
T("privacy", "error_reporting_off", "DISABLE WINDOWS ERROR REPORTING",
  "Stops Windows from collecting crash and hang reports for apps and sending them to Microsoft.",
  [R(r"HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting", "Disabled", 1)],
  "Keeps crash data (which can contain parts of documents in memory) on your PC.",
  "No report uploads or WerFault.exe report processing after crashes.",
  "Microsoft's 'solutions to problems' stop appearing and developers get no crash reports from you. Blue-screen dump files are still written locally.",
  "Everyone who doesn't rely on Windows' problem reports.", "none", "low", OC)
T("privacy", "advertising_id_policy", "DISABLE ADVERTISING ID (ALL USERS)",
  "Policy that turns off the advertising ID apps use to show personalised ads, for every account on this PC.",
  [R(POL + r"\Windows\AdvertisingInfo", "DisabledByGroupPolicy", 1)],
  "Stops apps from tracking you across apps with a shared ad identifier.", "Ads in apps become non-personalised.",
  "None beyond less relevant ads.", "Everyone.", "signout", "low", OC)
T("privacy", "tailored_experiences_off", "DISABLE TAILORED EXPERIENCES",
  "Policy that stops Microsoft from using your diagnostic data to personalise tips, ads and recommendations in Windows.",
  [R(r"HKCU\SOFTWARE\Policies\Microsoft\Windows\CloudContent", "DisableTailoredExperiencesWithDiagnosticData", 1)],
  "Prevents diagnostic data from being used for marketing.", "Fewer personalised promotions.",
  "Tips and suggestions become generic.", "Everyone.", "signout", "low", OC)
T("privacy", "inking_typing_off", "DISABLE INKING & TYPING PERSONALISATION",
  "Stops Windows from collecting your handwriting and typing patterns (and harvesting your contacts' names) to build a personal dictionary.",
  [R(r"HKCU\Software\Microsoft\InputPersonalization", "RestrictImplicitInkCollection", 1, d=0),
   R(r"HKCU\Software\Microsoft\InputPersonalization", "RestrictImplicitTextCollection", 1, d=0),
   R(r"HKCU\Software\Microsoft\InputPersonalization\TrainedDataStore", "HarvestContacts", 0, d=1)],
  "Keeps what you type and write out of Windows' personalisation data.", "Less typing data collected.",
  "Handwriting recognition and text suggestions learn less from you, so they may be less accurate.",
  "Anyone not using pen handwriting or touch keyboard suggestions.", "signout", "low", EX)
T("privacy", "typing_insights_off", "DISABLE TYPING INSIGHTS",
  "Turns off 'Typing insights' (Settings > Time & language > Typing), which records statistics about your typing and autocorrect use.",
  [R(r"HKCU\Software\Microsoft\Input\TIPC", "Enabled", 0, d=1)],
  "Stops typing statistics being collected.", "Less input data collected.",
  "The Typing insights page shows no data.", "Everyone.", "none", "low", EX)
T("privacy", "limit_diagnostic_logs", "LIMIT DIAGNOSTIC LOG COLLECTION (POLICY)",
  "Policy that stops Windows from collecting additional diagnostic logs when your diagnostic level is Optional.",
  [R(POL + r"\Windows\DataCollection", "LimitDiagnosticLogCollection", 1)],
  "Reduces the extra log files sent with optional diagnostic data.", "Less data uploaded.",
  "Microsoft gets less information to diagnose problems on your PC.", "Everyone.", "none", "low", EX)
T("privacy", "limit_dump_collection", "LIMIT CRASH-DUMP COLLECTION (POLICY)",
  "Policy that limits crash dumps sent with optional diagnostic data to small 'mini' dumps.",
  [R(POL + r"\Windows\DataCollection", "LimitDumpCollection", 1)],
  "Large memory dumps can contain document contents; this keeps them from being sent.", "Less sensitive data uploaded.",
  "Microsoft gets less detail about crashes.", "Everyone.", "none", "low", EX)
for lid, logger, title, what, cost in [
    ("diagtrack", "Diagtrack-Listener", "DIAGTRACK BOOT LOGGER", "the 'Diagtrack-Listener' trace session that starts at boot to buffer telemetry events for the Connected User Experiences (DiagTrack) service", "None for normal use; telemetry events from early boot are not collected."),
    ("sqm", "SQMLogger", "SQM LOGGER", "the 'SQMLogger' session that records Software Quality Metrics (legacy Customer Experience Improvement Program data)", "None for normal use."),
    ("setup_tel", "SetupPlatformTel", "SETUP TELEMETRY LOGGER", "the 'SetupPlatformTel' session that records telemetry about Windows setup and feature upgrades", "Upgrade problems are harder for Microsoft to diagnose."),
    ("oobe", "CloudExperienceHostOobe", "OOBE EXPERIENCE LOGGER", "the 'CloudExperienceHostOobe' session that logs the out-of-box and first-sign-in screens", "None for normal use."),
    ("wifi", "WiFiSession", "WI-FI SESSION TRACE", "the always-on 'WiFiSession' trace that records Wi-Fi connection events", "The Wi-Fi report (netsh wlan show wlanreport) and network troubleshooter have less history."),
    ("lwtnet", "LwtNetLog", "NETWORK LIGHTWEIGHT TRACE", "the always-on 'LwtNetLog' network diagnostics trace", "The network troubleshooter has less data to work with."),
]:
    T("privacy", "autologger_" + lid, "DISABLE " + title,
      f"Stops {what} from starting at boot (sets its Autologger 'Start' value to 0). The logger's registry key is left in place, so turning this off puts it back exactly.",
      [R(AUTOLOG + "\\" + logger, "Start", 0, d=1, must=True)],
      "Event-trace sessions run constantly from boot and write to disk; this one only serves diagnostics/telemetry.",
      "One less trace session and its small, constant disk writes.", cost,
      "Everyone who doesn't need Microsoft diagnostics.", "reboot", "low", OC)

# ============================ NETWORK OPTIMIZATION ===========================
T("network", "ncsi_probe_off", "DISABLE INTERNET CONNECTIVITY PROBES",
  "Stops Windows from contacting www.msftconnecttest.com each time the network changes to check whether it has internet access (policy plus service setting).",
  [R(POL + r"\Windows\NetworkConnectivityStatusIndicator", "NoActiveProbe", 1),
   R(r"HKLM\SYSTEM\CurrentControlSet\Services\NlaSvc\Parameters\Internet", "EnableActiveProbing", 0, d=1)],
  "Removes a phone-home request made every time you connect.", "No connectivity-check traffic to Microsoft.",
  "The network icon may show 'No internet' while you are online, captive-portal login pages (hotel/airport Wi-Fi) no longer open automatically, and apps that trust that status (Microsoft Store, Office, Spotify) can think they are offline.",
  "Privacy-focused desktop users on a home network.", "reboot", "medium", OC)
T("network", "delivery_optimization_p2p_off", "DISABLE PEER-TO-PEER UPDATE SHARING",
  "Policy that makes Delivery Optimization download Windows and Store updates from Microsoft only (HTTP mode), never from or to other PCs. Updates themselves keep working.",
  [R(POL + r"\Windows\DeliveryOptimization", "DODownloadMode", 0)],
  "Stops your PC from uploading update data to other computers.", "Your upload bandwidth is never used to serve updates.",
  "On a network with several Windows PCs each one downloads updates separately.",
  "Everyone, especially on metered or slow uploads.", "none", "low",
  OC)
T("network", "nic_eee_off", "DISABLE ENERGY-EFFICIENT ETHERNET",
  "Sets the 'Energy-Efficient Ethernet' (802.3az) advanced property to Disabled on every wired adapter whose driver exposes it.",
  [KEYS(NIC, "*EEE", "0", has="*SpeedDuplex")],
  "The link no longer idles in low-power mode between packets.",
  "Can remove small latency spikes and occasional link drops some switch/driver combinations show with EEE.",
  "Slightly higher power use (well under a watt per link). Adapters without the setting are left alone.",
  "Desktops on wired Ethernet.", "adapter", "low", OC)
T("network", "nic_flow_control_off", "DISABLE ETHERNET FLOW CONTROL",
  "Sets the 'Flow Control' advanced property to Disabled on wired adapters that expose it, so the adapter ignores and never sends Ethernet pause frames.",
  [KEYS(NIC, "*FlowControl", "0", has="*SpeedDuplex")],
  "Pause frames can briefly stall traffic; some players disable them for steadier latency.",
  "No pause-frame stalls.",
  "When a switch is congested the adapter may drop packets instead of pausing; TCP resends them, which can lower throughput on busy networks.",
  "Single-PC home networks; competitive players willing to test.", "adapter", "medium", OC)
T("network", "nic_interrupt_moderation_off", "DISABLE INTERRUPT MODERATION",
  "Sets 'Interrupt Moderation' to Disabled on wired adapters, so the adapter signals the CPU for every packet instead of batching them.",
  [KEYS(NIC, "*InterruptModeration", "0", has="*SpeedDuplex")],
  "Removes the adapter's batching delay.",
  "Packets reach the game a few microseconds sooner; visible in latency benchmarks, usually not noticeable in play.",
  "Many more CPU interrupts at high download speeds, which can reduce throughput or cost FPS on slower CPUs.",
  "Competitive players with fast CPUs who want to test it.", "adapter", "medium", OC,
  note="No guaranteed in-game improvement; measure before and after.")
T("network", "nic_wake_on_lan_off", "DISABLE WAKE-ON-LAN",
  "Disables 'Wake on Magic Packet' and 'Wake on Pattern Match' on wired adapters that expose them.",
  [KEYS(NIC, "*WakeOnMagicPacket", "0", has="*SpeedDuplex"), KEYS(NIC, "*WakeOnPattern", "0", has="*SpeedDuplex")],
  "Stops network traffic from waking the PC from sleep.", "No unexpected wake-ups caused by network activity.",
  "The PC can't be woken remotely (WoL tools, some NAS or backup schedules).",
  "Anyone who doesn't wake the PC over the network; laptops.", "adapter", "low", OC)
T("network", "nic_vendor_power_saving_off", "DISABLE NIC VENDOR POWER SAVING",
  "Turns off driver-specific power-saving features on adapters that expose them: Realtek 'Green Ethernet', 'Power Saving Mode', 'Gigabit Lite', 'Advanced EEE', Intel 'Auto Power Saver' and 'Ultra Low Power Mode'. Each adapter only gets the names its own driver uses.",
  [KEYS(NIC, n, "0", has="*SpeedDuplex") for n in ("EnableGreenEthernet", "PowerSavingMode", "GigaLite", "AdvancedEEE", "*NicAutoPowerSaver", "AutoPowerSaveModeEnabled", "ULPMode")],
  "These features renegotiate link speed or power down parts of the adapter when traffic is low.",
  "Avoids brief disconnects or a link that comes back at a lower speed.",
  "A little more power use; on laptops on battery that means shorter battery life.",
  "Desktops on wired Ethernet.", "adapter", "low", OC)
T("network", "nic_buffers_up", "LARGER NIC RECEIVE/TRANSMIT BUFFERS",
  "Raises the adapter's 'Receive Buffers' to 2048 and 'Transmit Buffers' to 1024 on wired adapters that expose them.",
  [KEYS(NIC, "*ReceiveBuffers", "2048", has="*SpeedDuplex"), KEYS(NIC, "*TransmitBuffers", "1024", has="*SpeedDuplex")],
  "Bigger rings give the adapter room to absorb traffic bursts.",
  "Fewer dropped packets during bursts such as large downloads while gaming.",
  "Uses a few MB more kernel memory. A driver whose maximum is lower normally clamps the value; in rare cases the adapter fails to start until you turn this OFF.",
  "Wired desktops with heavy network use.", "adapter", "medium", OC)
T("network", "nic_lso_off", "DISABLE LARGE SEND OFFLOAD",
  "Disables 'Large Send Offload V2' (IPv4 and IPv6) on wired adapters, so Windows splits outgoing TCP data itself instead of the adapter.",
  [KEYS(NIC, "*LsoV2IPv4", "0", has="*SpeedDuplex"), KEYS(NIC, "*LsoV2IPv6", "0", has="*SpeedDuplex")],
  "Some drivers have buggy LSO that causes upload stalls or latency spikes.",
  "Can fix slow or bursty uploads on affected drivers.",
  "More CPU work when uploading at high speed.", "Users with upload problems; testers.", "adapter", "medium", OC)
T("network", "nic_no_power_off", "DON'T LET WINDOWS TURN OFF THE ADAPTER",
  "Clears 'Allow the computer to turn off this device to save power' on wired adapters (PnPCapabilities = 24), the same as unticking it in Device Manager.",
  [KEYS(NIC, "PnPCapabilities", 24, k="dword", has="*SpeedDuplex", only_existing=False)],
  "Windows can power the adapter down when idle, which some adapters handle badly.",
  "Avoids 'network adapter disappeared' or slow-to-reconnect problems after idle or sleep.",
  "The adapter stays powered during sleep (slightly higher sleep power on laptops).",
  "Desktops; anyone who loses Ethernet after sleep.", "reboot", "low", OC)
T("network", "tcp_time_wait_30s", "SHORTEN TCP TIME_WAIT TO 30 SECONDS",
  "Closed TCP connections stay reserved for 30 s instead of the default 120 s (TcpTimedWaitDelay).",
  [R(r"HKLM\SYSTEM\CurrentControlSet\Services\Tcpip\Parameters", "TcpTimedWaitDelay", 30)],
  "Programs that open many short connections can run out of local ports.",
  "Ports are freed sooner for download managers, torrent clients and local servers.",
  "In theory a very late packet of an old connection could be mixed up with a new one; harmless in normal use.",
  "Heavy downloaders, developers running local servers.", "reboot", "low", EX)

# ==================================== DNS ===================================
T("dns", "doh_allow", "ALLOW DNS-OVER-HTTPS (AUTO-UPGRADE)",
  "Sets the 'Configure DNS over HTTPS name resolution' policy to Allow DoH. When your DNS server is one Windows knows supports DoH (Cloudflare 1.1.1.1, Google 8.8.8.8, Quad9 9.9.9.9 and others), lookups go encrypted over HTTPS; otherwise normal DNS is used.",
  [R(DNSPOL, "DoHPolicy", 2)],
  "Encrypts DNS lookups when possible without risking broken name resolution.",
  "Your ISP and others on the network can't read or alter lookups sent to supported servers.",
  "No effect while you use your ISP's DNS servers. Networks that filter DNS (school, work, parental controls) may stop seeing your lookups.",
  "Anyone using a public resolver - pairs with 'Use Cloudflare DNS' on the Network page.", "reboot", "low", EX,
  builds=W11, note="'Require DoH' is deliberately not offered: with a non-DoH server it breaks all name resolution.")
T("dns", "llmnr_off", "DISABLE LLMNR",
  "Policy that turns off Link-Local Multicast Name Resolution, the fallback that broadcasts a name on the local network when DNS can't answer.",
  [R(DNSPOL, "EnableMulticast", 0)],
  "LLMNR is used in well-known attacks to capture password hashes on shared networks.",
  "Closes that attack and stops broadcast name lookups.",
  "Old devices on the LAN reachable only by LLMNR name stop resolving by name; use their IP or DNS name.",
  "Everyone, especially laptops on public Wi-Fi.", "none", "low", EX)
T("dns", "smart_multihomed_off", "DISABLE SMART MULTI-HOMED NAME RESOLUTION",
  "Policy that stops Windows from sending each DNS query to every network adapter at once and taking the fastest answer.",
  [R(DNSPOL, "DisableSmartNameResolution", 1)],
  "With a VPN connected, queries otherwise also go to your normal ISP's DNS (a 'DNS leak').",
  "Lookups use one adapter's DNS in order, keeping VPN lookups inside the VPN.",
  "If the first adapter's DNS is slow or broken, Windows waits instead of racing another adapter.",
  "VPN users.", "none", "low", EX)
T("dns", "mdns_off", "DISABLE MULTICAST DNS (mDNS)",
  "Turns off Windows' built-in mDNS resolver/responder (names ending in .local).",
  [R(DNSCACHE, "EnableMDNS", 0)],
  "mDNS constantly announces and listens on the local network.", "Less multicast chatter; one less listening service.",
  "Devices that announce themselves via mDNS (some printers, Chromecast, smart-home hubs) may stop being found by Windows' own features; apps with their own mDNS (Chrome) are unaffected.",
  "Users without network printers/casting devices.", "reboot", "medium", EX)
T("dns", "negative_cache_off", "DON'T CACHE FAILED DNS LOOKUPS",
  "Windows normally remembers a failed lookup ('name not found') for up to 15 minutes; this sets that time to 0 (MaxNegativeCacheTtl).",
  [R(DNSCACHE, "MaxNegativeCacheTtl", 0)],
  "A temporarily failed lookup otherwise keeps failing until the cache entry expires.",
  "Sites that were briefly unreachable (after changing DNS or rebooting the router) resolve again immediately.",
  "Names that really don't exist are asked again every time (negligible traffic).",
  "Everyone.", "reboot", "low", EX)
T("dns", "netbios_off", "DISABLE NETBIOS OVER TCP/IP",
  "Sets NetBIOS over TCP/IP to Disabled on every network interface (the 'Disable NetBIOS over TCP/IP' option on the WINS tab).",
  [KEYS(r"HKLM\SYSTEM\CurrentControlSet\Services\NetBT\Parameters\Interfaces", "NetbiosOptions", 2, k="dword", sub=r"^Tcpip_", only_existing=False)],
  "NetBIOS name broadcasts are an old protocol and an attack surface (NBNS spoofing).",
  "No NetBIOS broadcasts; one less legacy protocol listening.",
  "Very old devices/NAS boxes reachable only by NetBIOS name, and SMB1-era network browsing, stop resolving. Modern file sharing by DNS name is unaffected. Interfaces added later keep the default.",
  "Home and laptop users without legacy network devices.", "reboot", "medium", EX)

# ============================== WINDOWS SECURITY ============================
T("security", "remote_assistance_off", "BLOCK REMOTE ASSISTANCE INVITATIONS",
  "Unticks 'Allow Remote Assistance connections to this computer' (System Properties > Remote).",
  [R(r"HKLM\SYSTEM\CurrentControlSet\Control\Remote Assistance", "fAllowToGetHelp", 0, d=1)],
  "Remote Assistance is a remote-control entry point often abused by tech-support scammers.",
  "Nobody can take control through a Remote Assistance invitation.",
  "You can't receive Remote Assistance (msra.exe) help. Quick Assist and Remote Desktop are separate.",
  "Everyone who doesn't use Remote Assistance.", "none", "low", OC)
T("security", "autorun_off", "DISABLE AUTORUN ON ALL DRIVES",
  "Policy that stops Windows from automatically running anything (autorun.inf) from USB sticks, discs or network drives.",
  [R(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer", "NoDriveTypeAutoRun", 255)],
  "Autorun is a classic way malware spreads through USB drives.", "Inserted media can't start programs by itself.",
  "Installer discs and some USB gadgets no longer start automatically; open them from File Explorer.",
  "Everyone.", "signout", "low", EX)
T("security", "wsh_off", "DISABLE WINDOWS SCRIPT HOST",
  "Stops .vbs, .vbe, .js and .wsf script files from running through wscript.exe/cscript.exe.",
  [R(r"HKLM\SOFTWARE\Microsoft\Windows Script Host\Settings", "Enabled", 0)],
  "Script files in emails and downloads are a common malware delivery method.", "Those scripts can't run.",
  "Old installers, login scripts or tools that use VBScript/JScript fail with 'Windows Script Host access is disabled'.",
  "Home users who don't run VBScript tools.", "none", "medium", EX)
T("security", "lsa_protection_on", "ENABLE LSA PROTECTION",
  "Runs the Local Security Authority (lsass.exe) as a protected process. Value 2 enables it without a UEFI lock, so it can be turned off again.",
  [R(r"HKLM\SYSTEM\CurrentControlSet\Control\Lsa", "RunAsPPL", 2)],
  "Credential-dumping tools read passwords and hashes from lsass memory.", "Stops non-protected processes from reading lsass memory.",
  "Third-party LSA plug-ins (some smart-card, VPN or password-sync tools) must be Microsoft-signed or they stop loading; check Event Viewer > CodeIntegrity after enabling.",
  "Everyone; already on by default on many new Windows 11 installs.", "reboot", "medium", EX, builds=[22621, 99999])
T("security", "uac_always_notify", "UAC: ALWAYS NOTIFY",
  "Raises User Account Control to the top level: you are also asked when Windows settings are changed, not only when apps request admin rights.",
  [R(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System", "ConsentPromptBehaviorAdmin", 2, d=5)],
  "Malware can silently use auto-elevating Windows tools to bypass the default UAC level.", "Closes those UAC bypasses.",
  "More UAC prompts when changing some Windows settings.", "Security-conscious users.", "none", "low", EX,
  note="Turning UAC off entirely (EnableLUA=0) is unsafe and is not offered here.")
T("security", "clear_pagefile_at_shutdown", "CLEAR PAGE FILE AT SHUTDOWN",
  "Windows overwrites the page file with zeros during shutdown.",
  [R(MM_KEY, "ClearPageFileAtShutdown", 1, d=0)],
  "Memory swapped to disk can contain passwords or documents.", "That data can't be recovered from the drive after shutdown.",
  "Shutdown takes longer (seconds to minutes depending on page-file size and drive). Not done for hibernate/Fast Startup.",
  "Shared or portable PCs without full-disk encryption.", "reboot", "low", EX)
T("security", "smb1_off", "REMOVE SMB 1.0 SUPPORT",
  "Disables the 'SMB 1.0/CIFS File Sharing Support' optional feature (client and server parts).",
  [FE("SMB1Protocol"), FE("SMB1Protocol-Client"), FE("SMB1Protocol-Server")],
  "SMB1 is the protocol exploited by WannaCry and NotPetya.", "Removes that attack surface.",
  "Very old NAS boxes, routers with USB sharing and Windows XP machines that only speak SMB1 can't be reached.",
  "Everyone (already off by default on clean Windows 11 installs).", "reboot", "low", OC)
T("security", "powershell_v2_off", "REMOVE POWERSHELL 2.0 ENGINE",
  "Disables the legacy Windows PowerShell 2.0 engine that still ships beside 5.1 on many installs.",
  [FE("MicrosoftWindowsPowerShellV2Root"), FE("MicrosoftWindowsPowerShellV2")],
  "Attackers start PowerShell 2.0 to bypass script-block logging and antivirus (AMSI) scanning.", "Closes that downgrade path.",
  "Ancient scripts that need -Version 2 stop working. Windows 11 24H2 removed the engine, so it shows as not supported there.",
  "Everyone.", "reboot", "low", OC)

# ============================== WINDOWS SERVICES ============================
SVC = [
    # id, service, title, default start (2 auto, 3 manual), desc, cost, risk, who, src
    ("wap_push", "dmwappushservice", "WAP PUSH MESSAGE ROUTING", 3, "Routes WAP push messages; Windows uses it mainly for device-management (MDM/Intune) enrollment and some diagnostics messaging.", "Work/school device enrollment (Intune/MDM) stops working.", "low", "Home PCs not managed by an organisation.", OC),
    ("diag_hub", "diagnosticshub.standardcollector.service", "DIAGNOSTICS HUB COLLECTOR", 3, "Collects real-time ETW traces for Visual Studio's diagnostic tools.", "Visual Studio's Diagnostic Tools and profiler can't collect data.", "low", "Non-developers.", OC),
    ("maps_broker", "MapsBroker", "DOWNLOADED MAPS MANAGER", 2, "Manages offline maps for the Maps app.", "Offline maps can't be downloaded or updated.", "low", "Everyone not using offline maps.", OC),
    ("geolocation", "lfsvc", "GEOLOCATION SERVICE", 3, "Provides the PC's location to apps.", "No app gets your location; Find my device and automatic time zone stop working.", "medium", "Desktops that never need location.", OC),
    ("retail_demo", "RetailDemo", "RETAIL DEMO SERVICE", 3, "Only used when a PC is put into store-display demo mode.", "None outside retail demo mode.", "low", "Everyone.", OC),
    ("wer", "WerSvc", "WINDOWS ERROR REPORTING SERVICE", 3, "Processes crash and hang reports.", "Crash reports aren't processed or sent; 'check for solutions' does nothing.", "low", "Everyone who doesn't use problem reports.", OC),
    ("wer_cpl", "wercplsupport", "PROBLEM REPORTS CONTROL PANEL SUPPORT", 3, "Backs the Control Panel 'Problem Reports' page.", "That page can't list or clear reports.", "low", "Everyone.", OC),
    ("pca", "PcaSvc", "PROGRAM COMPATIBILITY ASSISTANT", 2, "Watches programs you run and offers compatibility fixes when an old program fails or installs incorrectly.", "No automatic compatibility help for legacy software.", "low", "PCs running modern software.", OC),
    ("spooler", "Spooler", "PRINT SPOOLER", 2, "Queues and manages print jobs.", "No printing at all - including 'Microsoft Print to PDF' - and printer settings can't be opened.", "medium", "PCs that never print.", OC),
    ("fax", "Fax", "FAX SERVICE", 3, "Sends and receives faxes through a fax modem.", "Windows Fax and Scan can't fax.", "low", "Everyone without a fax modem.", EX),
    ("wmp_share", "WMPNetworkSvc", "WINDOWS MEDIA PLAYER NETWORK SHARING", 3, "Shares the Windows Media Player library with other devices.", "WMP Legacy can't stream its library to other devices.", "low", "Everyone.", OC),
    ("xbl_game_save", "XblGameSave", "XBOX LIVE GAME SAVE", 3, "Syncs saves for Xbox-enabled PC games.", "Games from the Xbox app/Game Pass can't sync saves to the cloud, so progress may differ between PCs.", "medium", "Players who don't use Xbox/Game Pass PC games.", OC),
    ("gamedvr_user", "BcastDVRUserService", "GAME DVR & BROADCAST USER SERVICE", 3, "Per-user service Game Bar uses to record clips and broadcast.", "Game Bar recording (Win+Alt+R) and clips stop working.", "low", "Players who record with other software.", OC),
    ("capture", "CaptureService", "CAPTURE SERVICE", 3, "Lets apps capture the screen through the Windows Graphics Capture API.", "Snipping Tool (Win+Shift+S), Game Bar capture and screen sharing in some apps stop working.", "medium", "Users of third-party capture tools only.", OC),
    ("clipboard", "cbdhsvc", "CLIPBOARD USER SERVICE", 3, "Provides clipboard history and clipboard sync.", "Clipboard history (Win+V), pinned items and cross-device clipboard stop. Normal copy & paste keeps working.", "medium", "Users who never use Win+V.", OC),
    ("phone", "PhoneSvc", "PHONE SERVICE", 3, "Manages telephony state on the device.", "Calls through Phone Link and some VoIP apps may stop working.", "low", "Users without Phone Link calling.", OC),
    ("tapi", "TapiSrv", "TELEPHONY (TAPI)", 3, "Telephony API for modems and telephony apps.", "Dial-up modems, fax and some older VoIP/CTI software stop; can also affect some VPN clients.", "medium", "PCs without modems or telephony software.", OC),
    ("payments_nfc", "SEMgrSvc", "PAYMENTS AND NFC/SE MANAGER", 3, "Manages NFC payments and secure elements.", "NFC payments stop (most desktops have no NFC).", "low", "Desktops.", OC),
    ("wallet", "WalletService", "WALLET SERVICE", 3, "Hosts objects used by Microsoft Wallet clients.", "Wallet clients can't store payment data (rarely used on PCs).", "low", "Everyone.", OC),
    ("messaging", "MessagingService", "MESSAGING SERVICE", 3, "Per-user service for text messaging (SMS relay).", "SMS/MMS in Phone Link may stop.", "low", "Users not texting through Phone Link.", OC),
    ("contact_index", "PimIndexMaintenanceSvc", "CONTACT DATA INDEXING", 3, "Indexes contacts for fast search in Mail/People.", "Contact search in Mail/People may be slower or incomplete.", "low", "Users of other mail apps.", OC),
    ("sync_host", "OneSyncSvc", "SYNC HOST", 2, "Per-user service that syncs mail, contacts and calendar for the built-in apps.", "Mail/Calendar/People and some Microsoft-account data stop syncing.", "medium", "Users of other mail/calendar apps.", OC),
    ("hotspot", "icssvc", "MOBILE HOTSPOT SERVICE", 3, "Lets the PC share its internet connection as a hotspot.", "Mobile Hotspot can't be turned on.", "low", "Desktops on Ethernet.", OC),
    ("lltd", "lltdsvc", "LINK-LAYER TOPOLOGY DISCOVERY MAPPER", 3, "Draws the network map.", "The network map feature has nothing to show.", "low", "Everyone.", OC),
    ("ssdp", "SSDPSRV", "SSDP DISCOVERY", 3, "Discovers UPnP devices (media servers, some printers, smart TVs) on the network.", "UPnP devices stop appearing in File Explorer > Network; casting to some DLNA TVs stops.", "medium", "Users without UPnP/DLNA devices.", OC),
    ("upnp_host", "upnphost", "UPNP DEVICE HOST", 3, "Lets this PC host UPnP devices for others.", "This PC can't act as a UPnP/DLNA device.", "low", "Everyone.", OC),
    ("fd_host", "fdPHost", "FUNCTION DISCOVERY PROVIDER HOST", 3, "Hosts network-discovery providers.", "Other PCs, printers and media devices stop appearing in File Explorer > Network.", "medium", "PCs that don't browse the local network.", OC),
    ("fd_publish", "FDResPub", "FUNCTION DISCOVERY RESOURCE PUBLICATION", 3, "Announces this PC and its shared resources on the network.", "Other PCs stop seeing this PC under Network.", "low", "PCs that don't share anything.", OC),
    ("ics", "SharedAccess", "INTERNET CONNECTION SHARING", 3, "Provides NAT and DHCP for connection sharing.", "Internet Connection Sharing and Mobile Hotspot stop; some VM/emulator networking that uses ICS fails.", "medium", "PCs without hotspot or VM use.", OC),
    ("netbios_helper", "lmhosts", "TCP/IP NETBIOS HELPER", 3, "Supports NetBIOS over TCP/IP name resolution.", "NetBIOS name resolution and old-style network browsing stop.", "low", "Users without legacy network devices.", OC),
    ("biometrics", "WbioSrvc", "WINDOWS BIOMETRIC SERVICE", 3, "Captures and matches fingerprint and face data for Windows Hello.", "Windows Hello fingerprint and face sign-in stop - make sure you know your PIN/password first.", "medium", "PCs without fingerprint/face sign-in.", OC),
    ("sensor", "SensorService", "SENSOR SERVICE", 3, "Manages device sensors.", "Auto-rotation, adaptive brightness and other sensor features on laptops/tablets stop.", "medium", "Desktops.", OC),
    ("sensor_monitor", "SensrSvc", "SENSOR MONITORING SERVICE", 3, "Monitors sensors such as the ambient-light sensor.", "Adaptive brightness from the light sensor stops.", "low", "Desktops.", OC),
    ("sensor_data", "SensorDataService", "SENSOR DATA SERVICE", 3, "Delivers data from sensors to apps.", "Apps can't read sensor data.", "low", "Desktops.", OC),
    ("parental", "WpcMonSvc", "PARENTAL CONTROLS", 3, "Enforces Microsoft Family Safety settings.", "Family Safety limits and activity reports aren't enforced on this PC.", "low", "PCs without child accounts.", OC),
    ("link_tracking", "TrkWks", "DISTRIBUTED LINK TRACKING CLIENT", 2, "Keeps shortcuts working when the files they point to move between NTFS volumes or network shares.", "Shortcuts to moved files may break instead of being repaired.", "low", "Everyone.", OC),
    ("wia", "stisvc", "WINDOWS IMAGE ACQUISITION (WIA)", 3, "Talks to scanners and some cameras.", "Scanners and some webcams/cameras that use WIA stop working.", "medium", "PCs without scanners.", OC),
    ("wia_events", "WiaRpc", "STILL IMAGE ACQUISITION EVENTS", 3, "Delivers scanner button events to apps.", "Pressing Scan on a device no longer starts an app.", "low", "PCs without scanners.", OC),
    ("alljoyn", "AJRouter", "ALLJOYN ROUTER", 3, "Routes AllJoyn messages for IoT devices.", "AllJoyn devices (rare, mostly old smart-home gear) can't talk to this PC.", "low", "Everyone.", OC),
    ("alg", "ALG", "APPLICATION LAYER GATEWAY", 3, "Plug-ins for protocols used through Internet Connection Sharing.", "Legacy protocol helpers for ICS (e.g. old FTP) stop.", "low", "Everyone.", OC),
    ("branchcache", "PeerDistSvc", "BRANCHCACHE", 3, "Caches content from servers for corporate networks.", "BranchCache stops (not present on Home).", "low", "Home users.", OC),
    ("peer_grouping", "p2psvc", "PEER NETWORKING GROUPING", 3, "Multi-party communication for Windows Peer Networking.", "Apps using Windows Peer Networking (rare) stop working.", "low", "Everyone.", OC),
    ("pnrp", "PNRPsvc", "PEER NAME RESOLUTION PROTOCOL", 3, "Serverless name resolution for peer networking.", "Apps using PNRP (rare) stop working.", "low", "Everyone.", OC),
    ("smart_card", "SCardSvr", "SMART CARD", 3, "Manages access to smart cards.", "Smart cards and security keys used as smart cards (PIV) stop working.", "medium", "PCs without smart cards.", OC),
    ("cert_propagation", "CertPropSvc", "CERTIFICATE PROPAGATION", 3, "Copies certificates from inserted smart cards to your certificate store.", "Smart-card certificates aren't added automatically.", "low", "PCs without smart cards.", OC),
    ("dps", "DPS", "DIAGNOSTIC POLICY SERVICE", 2, "Detects problems and runs troubleshooters; also feeds Task Manager's app history and network usage.", "Troubleshooters stop working and Task Manager's app history / data usage stop updating.", "medium", "Users who never use troubleshooters.", OC),
    ("diag_exec", "diagsvc", "DIAGNOSTIC EXECUTION SERVICE", 3, "Runs diagnostic scripts for troubleshooters.", "Troubleshooters that run scripts can't run.", "low", "Everyone.", OC),
    ("troubleshooting", "TroubleshootingSvc", "RECOMMENDED TROUBLESHOOTING", 3, "Runs Microsoft's recommended troubleshooters automatically.", "No automatic recommended troubleshooting.", "low", "Everyone.", OC),
    ("inventory", "InventorySvc", "INVENTORY AND COMPATIBILITY APPRAISAL", 3, "Collects an inventory of installed apps and drivers for upgrade-compatibility checks and telemetry.", "Feature-upgrade compatibility checks may be less accurate.", "low", "Everyone.", OC),
    ("health_experiences", "whesvc", "WINDOWS HEALTH AND OPTIMIZED EXPERIENCES", 3, "Provides 'health and optimized experience' recommendations.", "Those recommendations stop.", "low", "Everyone.", OC),
    ("gpu_perf_diag", "GraphicsPerfSvc", "GRAPHICS PERF DIAGNOSTIC", 3, "Collects GPU performance diagnostics for tools.", "GPU diagnostics for some tools stop; games are unaffected.", "low", "Everyone.", OC),
    ("file_sharing_server", "LanmanServer", "FILE & PRINTER SHARING SERVER", 2, "The 'Server' service that shares this PC's folders and printers over SMB.", "Nobody can open shared folders or printers on this PC and admin shares (C$) stop. Opening other PCs' shares still works.", "medium", "PCs that share nothing.", OC),
    ("auto_timezone", "tzautoupdate", "AUTO TIME ZONE UPDATER", 3, "Sets the time zone automatically from location.", "Time zone must be set by hand when travelling.", "low", "Desktops.", OC),
    ("nv_display_container", "NVDisplay.ContainerLocalSystem", "NVIDIA DISPLAY CONTAINER", 2, "Hosts NVIDIA Control Panel and tray components.", "NVIDIA Control Panel won't open and NVIDIA App overlay/tray features stop; the driver and games keep working.", "medium", "NVIDIA users who rarely open NVIDIA Control Panel.", OC),
    ("nv_container", "NvContainerLocalSystem", "NVIDIA LOCALSYSTEM CONTAINER", 2, "Runs NVIDIA App / GeForce Experience background components.", "NVIDIA App/GeForce Experience features (overlay, recording, driver update checks) stop.", "medium", "NVIDIA users without NVIDIA App.", OC),
    ("intel_lms", "LMS", "INTEL MANAGEMENT ENGINE LOCAL MANAGEMENT", 2, "Intel AMT/vPro local management service.", "Intel AMT remote management (business PCs) stops; consumer PCs don't use it.", "low", "Home PCs with Intel chipsets.", OC),
    ("offline_files", "CscService", "OFFLINE FILES", 3, "Keeps offline copies of network files (Sync Center).", "Offline Files/Sync Center stop working.", "low", "PCs not using Offline Files.", OC),
    ("beep", "beep", "PC SPEAKER BEEP DRIVER", 1, "Kernel driver for the motherboard speaker beep.", "No motherboard-speaker beeps from Windows.", "low", "Everyone.", OC),
    ("firewire", "1394ohci", "FIREWIRE (IEEE 1394) CONTROLLER", 3, "Driver for FireWire controllers.", "FireWire devices (old cameras, audio interfaces) stop working.", "low", "PCs without FireWire devices.", OC),
]
for sid, name, title, d, desc, cost, risk, who, src in SVC:
    T("services", "svc_" + sid, title,
      f"{desc} This tweak sets the '{name}' service to Disabled.",
      [S(name, d)],
      "One less service that can start in the background.",
      "Slightly less memory and background activity; in most cases a small, not measurable, gain.",
      cost, who, "reboot", risk, src if src != OC else OC)

# =============================== SCHEDULED TASKS ============================
AE = r"\Microsoft\Windows\Application Experience" + "\\"
TASKS = [
    ("compat_appraiser", AE + "Microsoft Compatibility Appraiser", "COMPATIBILITY APPRAISER", "Scans installed programs daily and sends compatibility telemetry; can use noticeable CPU and disk while it runs.", "Feature upgrades are offered without checking your apps first.", "low"),
    ("program_data_updater", AE + "ProgramDataUpdater", "PROGRAM DATA UPDATER", "Collects program telemetry for the Customer Experience Improvement Program (removed in newer builds).", "None.", "low"),
    ("mare_backup", AE + "MareBackup", "COMPATIBILITY DATA BACKUP", "Backs up compatibility-appraiser data for telemetry.", "None.", "low"),
    ("startup_app_task", AE + "StartupAppTask", "STARTUP APP IMPACT SCAN", "Measures how much each startup app slows sign-in.", "Task Manager's 'Startup impact' column may show 'Not measured'.", "low"),
    ("autochk_proxy", r"\Microsoft\Windows\Autochk\Proxy", "AUTOCHK SQM PROXY", "Collects and uploads quality metrics from Autochk.", "None.", "low"),
    ("ceip_consolidator", r"\Microsoft\Windows\Customer Experience Improvement Program\Consolidator", "CEIP CONSOLIDATOR", "Collects and sends Customer Experience Improvement Program data.", "None.", "low"),
    ("ceip_usb", r"\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip", "USB CEIP", "Collects USB usage statistics for Microsoft.", "None.", "low"),
    ("disk_diag", r"\Microsoft\Windows\DiskDiagnostic\Microsoft-Windows-DiskDiagnosticDataCollector", "DISK DIAGNOSTIC DATA COLLECTOR", "Collects disk diagnostic information and reports it to Microsoft.", "Windows' own disk-failure diagnostics have less data; SMART tools are unaffected.", "low"),
    ("siuf_dmclient", r"\Microsoft\Windows\Feedback\Siuf\DmClient", "FEEDBACK DATA CLIENT", "Downloads and runs feedback/survey scenarios.", "None.", "low"),
    ("siuf_scenario", r"\Microsoft\Windows\Feedback\Siuf\DmClientOnScenarioDownload", "FEEDBACK SCENARIO DOWNLOAD", "Runs feedback scenarios when they are downloaded.", "None.", "low"),
    ("wer_queue", r"\Microsoft\Windows\Windows Error Reporting\QueueReporting", "QUEUED ERROR REPORTING", "Sends queued crash reports.", "Queued crash reports are not sent.", "low"),
    ("maps_update", r"\Microsoft\Windows\Maps\MapsUpdateTask", "MAPS UPDATE", "Checks for offline map updates.", "Offline maps aren't updated automatically.", "low"),
    ("maps_toast", r"\Microsoft\Windows\Maps\MapsToastTask", "MAPS NOTIFICATIONS", "Shows notifications about offline maps.", "None.", "low"),
    ("net_trace", r"\Microsoft\Windows\NetTrace\GatherNetworkInfo", "GATHER NETWORK INFO", "Collects network configuration for diagnostics.", "Network diagnostics have less history.", "low"),
    ("power_efficiency", r"\Microsoft\Windows\Power Efficiency Diagnostics\AnalyzeSystem", "POWER EFFICIENCY DIAGNOSTICS", "Analyses power use and reports it.", "None.", "low"),
    ("device_census", r"\Microsoft\Windows\Device Information\Device", "DEVICE CENSUS", "Collects hardware and OS details for telemetry.", "None.", "low"),
    ("device_census_user", r"\Microsoft\Windows\Device Information\Device User", "DEVICE CENSUS (USER)", "Collects per-user device information for telemetry.", "None.", "low"),
    ("xbl_save_task", r"\Microsoft\Windows\XblGameSave\XblGameSaveTask", "XBOX GAME SAVE SYNC", "Syncs Xbox game saves in the background.", "Xbox/Game Pass cloud saves may not sync while games are closed.", "medium"),
    ("family_monitor", r"\Microsoft\Windows\Shell\FamilySafetyMonitor", "FAMILY SAFETY MONITOR", "Initialises Family Safety monitoring.", "Family Safety activity monitoring stops.", "low"),
    ("family_refresh", r"\Microsoft\Windows\Shell\FamilySafetyRefreshTask", "FAMILY SAFETY REFRESH", "Syncs Family Safety settings.", "Family Safety settings don't refresh.", "low"),
    ("sqm_tasks", r"\Microsoft\Windows\PI\Sqm-Tasks", "SQM TASKS", "Uploads Software Quality Metrics.", "None.", "low"),
]
# tasks renamed or moved in newer builds: every known location is listed, whichever exists is used
ALIASES = {
    "compat_appraiser": [AE + "Microsoft Compatibility Appraiser Exp"],
    "xbl_save_task": [r"\Microsoft\XblGameSave\XblGameSaveTask"],
}
for tid, path, title, desc, cost, risk in TASKS:
    T("tasks", "task_" + tid, title,
      f"{desc} This tweak disables the scheduled task '{path}'; turning it off re-enables the task if it was enabled before.",
      [TK(x) for x in [path] + ALIASES.get(tid, [])],
      "Stops one background task from waking up on a schedule.",
      "Fewer scheduled background runs (CPU/disk/network) - usually a small gain.",
      cost, "Everyone who doesn't need it.", "none", risk,
      OC)

# ============================ WINDOWS DEBLOATING ============================
T("debloat", "toast_notifications_off", "DISABLE APP NOTIFICATION BANNERS",
  "Turns off the main 'Notifications' switch (Settings > System > Notifications), so apps no longer show pop-up banners.",
  [R(r"HKCU\Software\Microsoft\Windows\CurrentVersion\PushNotifications", "ToastEnabled", 0, d=1)],
  "Removes interruptions from every app at once.", "No pop-up banners during games or work.",
  "You miss alerts from mail, chat and calendar apps unless you open them.", "Users who check apps themselves.", "none", "low", OC)
T("debloat", "silent_app_installs_off", "STOP SILENT INSTALLS OF SUGGESTED APPS",
  "Stops the Content Delivery Manager from quietly installing promoted apps and games (Candy Crush, TikTok and similar) for your account.",
  [R(CDM, n, 0, d=1) for n in ("SilentInstalledAppsEnabled", "PreInstalledAppsEnabled", "OemPreInstalledAppsEnabled", "PreInstalledAppsEverEnabled")],
  "Prevents unwanted apps from appearing in Start.", "No surprise app installs.",
  "None - apps you install yourself are unaffected.", "Everyone.", "signout", "low", OC)
T("debloat", "start_suggestions_off", "HIDE APP SUGGESTIONS IN START",
  "Turns off 'Show suggestions occasionally in Start' (promoted apps).",
  [R(CDM, "SubscribedContent-338388Enabled", 0, d=1), R(CDM, "SystemPaneSuggestionsEnabled", 0, d=1)],
  "Removes adverts from Start.", "Cleaner Start menu.", "None.", "Everyone.", "explorer", "low", OC)
T("debloat", "windows_tips_off", "DISABLE 'TIPS AND SUGGESTIONS' NOTIFICATIONS",
  "Turns off 'Get tips and suggestions when using Windows'.",
  [R(CDM, "SubscribedContent-338389Enabled", 0, d=1)],
  "Stops tip notifications.", "Fewer notifications.", "None.", "Everyone.", "none", "low", OC)
T("debloat", "lock_screen_tips_off", "HIDE FUN FACTS & TIPS ON THE LOCK SCREEN",
  "Removes the 'fun facts, tips, tricks and more' text and links shown on the lock screen.",
  [R(CDM, "SubscribedContent-338387Enabled", 0, d=1), R(CDM, "RotatingLockScreenOverlayEnabled", 0, d=1)],
  "Keeps the lock screen clean.", "No promotional text on the lock screen.", "None.", "Everyone.", "none", "low", OC)
T("debloat", "finish_setup_prompt_off", "DISABLE 'FINISH SETTING UP YOUR DEVICE' SCREEN",
  "Stops the full-screen 'Let's finish setting up your device' / 'Get even more out of Windows' prompt after updates.",
  [R(r"HKCU\Software\Microsoft\Windows\CurrentVersion\UserProfileEngagement", "ScoobeSystemSettingEnabled", 0)],
  "Skips an upsell screen after updates.", "Straight to the desktop after updates.", "None.", "Everyone.", "none", "low", OC)
T("debloat", "consumer_features_off", "DISABLE MICROSOFT CONSUMER EXPERIENCES",
  "Policy 'Turn off Microsoft consumer experiences': stops Windows from adding third-party app suggestions and promoted apps for users.",
  [R(POL + r"\Windows\CloudContent", "DisableWindowsConsumerFeatures", 1)],
  "Blocks promoted app installs at the policy level.", "Fewer promoted apps for new accounts.",
  "None.", "Everyone.", "signout", "low", OC,
  note="Microsoft documents this policy as fully effective on Enterprise/Education; on Home/Pro some promotions can still appear.")
T("debloat", "block_device_coinstallers", "BLOCK DEVICE COMPANION SOFTWARE AUTO-INSTALL",
  "Stops driver packages from running vendor 'co-installers' - the mechanism that silently installs apps like Razer Synapse or Logitech G HUB when you plug a device in.",
  [R(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Device Installer", "DisableCoInstallers", 1)],
  "Keeps vendor software from being installed without asking.", "No surprise companion apps when plugging in devices.",
  "Some devices may install with reduced features until you install the vendor software yourself.",
  "Users who install vendor software manually.", "none", "low", OC)
T("debloat", "no_driver_search_wu", "DON'T SEARCH WINDOWS UPDATE FOR DEVICE DRIVERS",
  "Device installation stops searching Windows Update for drivers (Device installation settings > No).",
  [R(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\DriverSearching", "SearchOrderConfig", 0, d=1)],
  "Windows can replace a driver you installed with an older or generic one.", "Your chosen GPU/audio drivers stay in place when devices are re-detected.",
  "New hardware may install with a basic driver until you install the vendor's; driver fixes then come only from vendors.",
  "Users who manage their own drivers (GPU especially).", "none", "medium", OC)
T("debloat", "exclude_wu_drivers", "EXCLUDE DRIVERS FROM WINDOWS UPDATE",
  "Policy 'Do not include drivers with Windows Updates': Windows Update keeps installing Windows updates but no longer offers driver updates.",
  [R(POL + r"\Windows\WindowsUpdate", "ExcludeWUDriversInQualityUpdate", 1)],
  "Stops Windows Update from overwriting drivers you chose.", "No unexpected driver changes after Patch Tuesday.",
  "You must update drivers (including security fixes) yourself from device vendors.",
  "Users who manage their own drivers.", "none", "medium", EX)
T("debloat", "device_metadata_off", "DON'T DOWNLOAD DEVICE METADATA",
  "Policy that stops Windows downloading manufacturer device info, icons and companion-app links for connected devices.",
  [R(POL + r"\Windows\Device Metadata", "PreventDeviceMetadataFromNetwork", 1)],
  "Avoids background downloads and app promotions for devices.", "No device metadata downloads.",
  "Devices and Printers shows generic icons and names.", "Everyone.", "none", "low", EX)
T("debloat", "meet_now_off", "HIDE MEET NOW (WINDOWS 10)",
  "Removes the Skype 'Meet Now' icon from the Windows 10 taskbar.",
  [R(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Policies\Explorer", "HideSCAMeetNow", 1)],
  "Removes an unused taskbar icon.", "Cleaner taskbar.", "None.", "Windows 10 users.", "explorer", "low", OC, builds=W10)
T("debloat", "news_interests_off_w10", "DISABLE NEWS AND INTERESTS (WINDOWS 10)",
  "Policy that turns off the Windows 10 taskbar weather/news feed.",
  [R(POL + r"\Windows\Windows Feeds", "EnableFeeds", 0)],
  "Removes a feed that loads web content in the background.", "No news process in the background.",
  "No weather on the taskbar.", "Windows 10 users.", "explorer", "low", OC, builds=W10)
T("debloat", "hide_start_recommended", "HIDE THE START 'RECOMMENDED' SECTION",
  "Hides the whole 'Recommended' area of the Windows 11 Start menu using the HideRecommendedSection policy. It also sets the 'education environment' flag, without which Home/Pro ignore the policy.",
  [R(POL + r"\Windows\Explorer", "HideRecommendedSection", 1),
   R(r"HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\Start", "HideRecommendedSection", 1),
   R(r"HKLM\SOFTWARE\Microsoft\PolicyManager\current\device\Education", "IsEducationEnvironment", 1)],
  "Start shows only pinned apps.", "No recent files or recommended apps in Start.",
  "The device is flagged as an education environment, which can also change a few other Windows defaults meant for schools.",
  "Users who want a minimal Start menu.", "explorer", "medium", OC, builds=W11)
T("debloat", "lock_screen_off", "SKIP THE LOCK SCREEN",
  "Policy that goes straight to the sign-in screen (PIN/password) instead of showing the picture lock screen first. Fully reversible: nothing is deleted.",
  [R(POL + r"\Windows\Personalization", "NoLockScreen", 1)],
  "One less screen to dismiss.", "Faster to the sign-in box.",
  "No lock-screen picture, clock, notifications or widgets; you still have to sign in.",
  "Desktop users.", "none", "low", OC,
  note="Microsoft documents this policy for Enterprise/Education; most Home/Pro builds honour it, some ignore it.")
T("debloat", "notepad_ai_off", "DISABLE AI FEATURES IN NOTEPAD",
  "Policy that turns off Copilot-powered features (Rewrite, Summarize) in the new Notepad.",
  [R(r"HKLM\SOFTWARE\Policies\WindowsNotepad", "DisableAIFeatures", 1)],
  "Keeps Notepad a plain editor.", "No AI buttons in Notepad.", "No AI rewrite/summarize in Notepad.",
  "Everyone not using Notepad AI.", "app", "low", EX, builds=W11)
T("debloat", "start_account_notifications_off", "HIDE ACCOUNT NOTIFICATIONS IN START",
  "Stops Start from showing Microsoft-account prompts such as 'Back up your files' or 'Sign in' badges.",
  [R(ADV, "Start_AccountNotifications", 0)],
  "Removes upsell badges from Start.", "Cleaner Start menu.", "None.", "Everyone.", "explorer", "low", EX, builds=[22621, 99999])

# ======================= MICROSOFT EDGE DEBLOAT (policies) ==================
for eid, val, title, desc, gain, cost in [
    ("shopping_off", "EdgeShoppingAssistantEnabled", "DISABLE SHOPPING ASSISTANT", "Removes Edge's price-comparison and coupon pop-ups.", "No shopping pop-ups.", "No price comparisons in Edge."),
    ("sidebar_off", "HubsSidebarEnabled", "HIDE THE SIDEBAR", "Removes the right-hand sidebar (Copilot, tools, games) from Edge.", "More room and fewer sidebar processes.", "No Copilot/sidebar apps in Edge."),
    ("startup_boost_off", "StartupBoostEnabled", "DISABLE STARTUP BOOST", "Stops Edge from launching hidden processes at sign-in so it opens faster.", "Fewer background Edge processes and less memory use after sign-in.", "Edge opens a little slower the first time."),
    ("recommendations_off", "ShowRecommendationsEnabled", "DISABLE TIPS & RECOMMENDATIONS", "Stops Edge showing feature recommendations and notifications.", "Fewer Edge pop-ups.", "None."),
    ("personalization_off", "PersonalizationReportingEnabled", "DISABLE BROWSING-DATA PERSONALISATION", "Stops Edge sending browsing history to Microsoft to personalise ads, news and other services.", "Less browsing data shared.", "Less personalised Microsoft news/ads."),
    ("feedback_off", "UserFeedbackAllowed", "DISABLE FEEDBACK PROMPTS", "Removes Edge's 'Send feedback' prompts and smile button.", "No feedback prompts.", "You can't send feedback from Edge."),
    ("newtab_content_off", "NewTabPageContentEnabled", "HIDE NEW TAB NEWS FEED", "Hides the Microsoft Start news feed on Edge's New Tab page.", "Lighter, faster New Tab page.", "No news on New Tab."),
]:
    T("edge", "edge_" + eid, title, desc + " (policy " + val + " = 0)", [R(EDGE, val, 0)],
      "Removes Edge extras you may not want.", gain, cost + " " + EDGE_NOTE, "Edge users.", "app", "low", EX)
T("edge", "edge_first_run_off", "SKIP FIRST-RUN SETUP", "Hides Edge's first-run setup pages (policy HideFirstRunExperience = 1).",
  [R(EDGE, "HideFirstRunExperience", 1)], "Skips setup screens.", "Edge opens straight to a page.",
  "You don't get the import/sign-in wizard. " + EDGE_NOTE, "Edge users.", "app", "low", EX)
T("edge", "edge_desktop_shortcut_off", "NO EDGE DESKTOP SHORTCUT AFTER UPDATES",
  "Policy that stops Edge updates from re-creating the Edge shortcut on the desktop.",
  [R(r"HKLM\SOFTWARE\Policies\Microsoft\EdgeUpdate", "CreateDesktopShortcutDefault", 0)],
  "Stops a deleted shortcut from coming back.", "Clean desktop.", "None.", "Everyone.", "none", "low", EX)

# =================================== GAMING =================================
T("gaming", "background_recording_off", "DISABLE BACKGROUND RECORDING",
  "Turns off Game Bar's 'Record what happened' (continuously records the last moments of gameplay).",
  [R(r"HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR", "HistoricalCaptureEnabled", 0)],
  "Background recording keeps the GPU's video encoder busy during play.",
  "Frees GPU encoder time and a little CPU/RAM while gaming.",
  "You can't save a clip of something that just happened.", "Players who don't use Game Bar clips.", "none", "low", EX)
T("gaming", "app_capture_off", "DISABLE GAME BAR CAPTURES (YOUR ACCOUNT)",
  "Turns off the per-user capture switch so Game Bar can't record or screenshot games. The existing 'Disable Game DVR' tweak is the all-users policy.",
  [R(r"HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR", "AppCaptureEnabled", 0), R(r"HKCU\System\GameConfigStore", "GameDVR_Enabled", 0, d=1)],
  "Prevents Game Bar capture hooks for your account.", "No capture overhead from Game Bar.",
  "Game Bar recording, screenshots and clips stop working for your account.", "Players using other capture software.", "none", "low", OC)
T("gaming", "windowed_game_optimizations", "OPTIMIZATIONS FOR WINDOWED GAMES",
  "Turns on 'Optimizations for windowed games' (Settings > Display > Graphics), which moves older DirectX 10/11 games running windowed or borderless to the modern flip presentation model.",
  [R(r"HKCU\Software\Microsoft\DirectX\UserGpuPreferences", "DirectXUserGlobalSettings", "SwapEffectUpgradeEnable=1;", "sz")],
  "Older DX10/11 games in borderless mode otherwise use a slower presentation path.",
  "Lower input latency in borderless windowed mode, and Auto HDR/VRR support for those games.",
  "Rare issues with overlays or capture tools in a few games (you can opt a game out in Graphics settings). This value holds all global graphics options, so it is replaced as a whole and restored as a whole when turned off.",
  "Players of older DX10/11 games in borderless mode.", "app", "low", OC, builds=[22621, 99999])
T("gaming", "global_timer_resolution", "ALLOW GLOBAL TIMER RESOLUTION REQUESTS",
  "Since Windows 10 2004 a program's request for a finer system timer only affects that program. On Windows 11 this value restores the old behaviour where such a request applies system-wide.",
  [R(r"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel", "GlobalTimerResolutionRequests", 1)],
  "Timer-resolution utilities rely on it to lower the timer for every program.",
  "Some players report steadier frame pacing with a 0.5 ms timer; results vary by game and PC.",
  "Higher idle power use and shorter battery life while any program requests a fine timer. No effect unless something requests one.",
  "Desktop players who use a timer-resolution tool.", "reboot", "medium", OC, builds=W11,
  note="No guaranteed FPS or latency improvement.")

# =============================== CPU & PROCESSOR ============================
T("cpu", "epp_performance_ac", "FAVOUR PERFORMANCE (EPP 0, PLUGGED IN)",
  "Sets 'Processor energy performance preference policy' to 0 % for the active power plan on AC power. Modern Intel and AMD CPUs use this hint to decide how quickly to raise clocks.",
  [PW(PROC, "36687f9e-e3a5-4dbf-b1dc-15eb381c6863", 0)],
  "The CPU ramps clocks up faster and holds them under light load.", "Snappier response; fewer clock ramp-up delays.",
  "Higher idle power and temperatures, more fan noise. The battery value is left alone.",
  "Desktops and plugged-in laptops.", "none", "low", EX,
  note="Applies to the plan active when you turn it on; switching to another plan shows this tweak as OFF.")
T("cpu", "hybrid_prefer_pcores", "HYBRID CPUs: PREFER PERFORMANCE CORES",
  "Sets the documented 'Heterogeneous thread scheduling policy' and its short-running-thread counterpart to 'Prefer performant processors' for the active plan on AC power.",
  [PW(PROC, "93b8b6dc-0698-4d1c-9ee4-0644e900c85d", 2), PW(PROC, "bae08b81-2d5e-4688-ad6a-13243356654b", 2)],
  "On CPUs with P-cores and E-cores the scheduler may put foreground work on slower E-cores.",
  "Threads go to P-cores first when they are free.",
  "More power use; background work also lands on P-cores. No effect on CPUs without E-cores.",
  "Intel 12th-gen and newer hybrid CPUs, plugged in.", "none", "low",
  EX)

# ================================ RAM & MEMORY ==============================
T("memory", "sysmain_off", "DISABLE SYSMAIN (SUPERFETCH)",
  "Stops the service that preloads frequently used apps into RAM and tunes memory and boot prefetching.",
  [S("SysMain", 2)],
  "Some users see heavy disk activity from SysMain right after boot.", "Less background disk and memory activity.",
  "Frequently used apps can start slower, especially from a hard disk; boot prefetching stops. Memory compression can no longer be changed while it is off.",
  "PCs with fast SSDs and plenty of RAM.", "reboot", "medium", OC)
T("memory", "svchost_grouping", "GROUP SVCHOST SERVICES",
  "Raises the RAM threshold above which Windows runs each service in its own svchost.exe, so services are grouped into fewer processes again (as on low-RAM PCs).",
  [R(r"HKLM\SYSTEM\CurrentControlSet\Control", "SvcHostSplitThresholdInKB", 0xFFFFFFFF, d=3670016)],
  "Dozens of svchost.exe processes each carry some overhead.", "Fewer processes and slightly lower memory use.",
  "If one service crashes it can take the others in its svchost with it, and Task Manager can't show per-service CPU use as precisely.",
  "Users who want fewer processes.", "reboot", "medium", OC)
T("memory", "memory_compression_off", "DISABLE MEMORY COMPRESSION",
  "Turns off compression of rarely used memory pages (the 'Memory Compression' process).",
  [MMA("MemoryCompression")],
  "Compressing and decompressing pages costs CPU time.", "Saves that CPU time.",
  "With less than about 16 GB of RAM, Windows pages to disk sooner, which is far slower than compression. Needs the SysMain service running to change.",
  "PCs with 32 GB or more RAM.", "reboot", "medium", EX)
T("memory", "page_combining_off", "DISABLE PAGE COMBINING",
  "Stops Windows from periodically scanning RAM for identical pages to merge them.",
  [MMA("PageCombining")],
  "The scan is periodic background work.", "Removes that background scan.",
  "Slightly higher memory use.", "PCs with plenty of RAM.", "reboot", "low", EX)
T("memory", "app_prelaunch_off", "DISABLE APP PRE-LAUNCH",
  "Stops Windows from starting Store apps (Mail, Photos and others) suspended in the background so they open faster.",
  [MMA("ApplicationPreLaunch")],
  "Pre-launched apps sit in memory even if you never open them.", "Fewer suspended apps in memory.",
  "Those apps open a bit slower.", "Users who rarely use Store apps.", "reboot", "low", EX)
T("memory", "paging_executive_off", "KEEP KERNEL & DRIVERS IN RAM",
  "Prevents Windows from paging kernel-mode code and drivers out to the page file.",
  [R(MM_KEY, "DisablePagingExecutive", 1, d=0)],
  "Paged-out kernel code must be read back from disk when needed.", "Avoids rare stalls after the PC has been idle.",
  "A few hundred MB less RAM for apps; little measurable benefit on PCs with plenty of RAM.",
  "PCs with 16 GB or more RAM.", "reboot", "low", EX)

# ================================ STORAGE & SSD =============================
T("storage", "long_paths_on", "ENABLE LONG FILE PATHS",
  "Lets programs that support it use paths longer than 260 characters.",
  [R(FS, "LongPathsEnabled", 1, d=0)],
  "Fixes 'path too long' errors.", "Deep folder trees, npm/git projects and archives work.",
  "Programs without long-path support still fail on such paths; nothing breaks.", "Developers; everyone.", "reboot", "low", OC)
T("storage", "ntfs_last_access_off", "DISABLE NTFS LAST-ACCESS TIMESTAMPS",
  "Stops NTFS updating each file's 'last accessed' time when it is read (0x80000001 = set by the user, disabled).",
  [R(FS, "NtfsDisableLastAccessUpdate", 0x80000001, d=0x80000002)],
  "Every file read otherwise causes a small metadata write.", "Fewer writes when reading many files.",
  "Tools that rely on 'last accessed' dates (e.g. Storage Sense's 'not opened for X days') see stale dates. Windows already disables it on large volumes by default.",
  "Everyone.", "reboot", "low", EX)
T("storage", "ntfs_8dot3_off", "STOP CREATING 8.3 SHORT FILE NAMES",
  "New files no longer get an extra DOS-style short name (like PROGRA~1). Existing short names stay.",
  [R(FS, "NtfsDisable8dot3NameCreation", 1, d=2)],
  "Short names are generated for every new file.", "Faster file creation in folders with many similar names.",
  "Very old 16-bit/DOS-era installers and tools that need short names may fail on new files.", "Everyone.", "reboot", "low", EX)
T("storage", "ntfs_memory_more", "MORE NTFS CACHE MEMORY",
  "Raises the paged-pool memory NTFS may use for its metadata cache (NtfsMemoryUsage = 2).",
  [R(FS, "NtfsMemoryUsage", 2)],
  "NTFS metadata is cached in paged pool.", "Can help with very large numbers of files (big repos, game assets).",
  "More kernel memory use; little effect on typical use.", "PCs with lots of RAM and many files.", "reboot", "low", EX)
T("storage", "thumbs_db_network_off", "NO THUMBS.DB ON NETWORK FOLDERS",
  "Stops File Explorer writing hidden thumbs.db thumbnail caches into network shares.",
  [R(r"HKCU\Software\Policies\Microsoft\Windows\Explorer", "DisableThumbsDBOnNetworkFolders", 1)],
  "thumbs.db files clutter shares and lock folders.", "No stray thumbs.db files and no 'file in use' when deleting network folders.",
  "Thumbnails for network files are generated again each visit.", "Users of NAS/network shares.", "explorer", "low", EX)

# ============================== POWER MANAGEMENT ============================
T("power", "pcie_aspm_off_ac", "PCI EXPRESS LINK POWER SAVING OFF (PLUGGED IN)",
  "Sets 'Link State Power Management' to Off for the active plan on AC power, so PCIe links (GPU, NVMe, network) never drop into low-power states.",
  [PW("501a4d13-42af-4429-9fd1-a8218c268e20", "ee12f906-d277-404b-b6da-e5fa1a576df5", 0)],
  "Waking a PCIe link from a low-power state adds a small delay.", "Removes those wake-up delays.",
  "A few watts more at idle.", "Desktops; plugged-in laptops.", "none", "low",
  EX)

# ============================ USB & PERIPHERALS =============================
T("usb", "usb3_lpm_off_ac", "USB 3 LINK POWER MANAGEMENT OFF (PLUGGED IN)",
  "Sets 'USB 3 Link Power Management' to Off for the active plan on AC power.",
  [PW("2a737441-1930-4402-8d77-b2bebba308a3", "d4e98f31-5ffe-4ce1-be31-1b38b384c009", 0)],
  "Some USB 3 devices stutter or disconnect when the link sleeps.",
  "Steadier USB audio interfaces, capture cards, external SSDs and wireless dongles.",
  "Slightly more power use.", "Desktops with USB audio/capture devices.", "none", "low", EX)
T("usb", "filter_keys_hotkey_off", "DISABLE FILTER KEYS SHORTCUT",
  "Stops holding Right Shift for 8 seconds from turning on Filter Keys (which makes the keyboard ignore quick or repeated presses).",
  [R(r"HKCU\Control Panel\Accessibility\Keyboard Response", "Flags", "122", "sz", d="126")],
  "The shortcut triggers by accident in games.", "No surprise accessibility pop-up or sluggish keyboard mid-game.",
  "The shortcut can't turn Filter Keys on (Settings still can).", "Gamers.", "signout", "low", OC)
T("usb", "toggle_keys_hotkey_off", "DISABLE TOGGLE KEYS SHORTCUT",
  "Stops holding Num Lock for 5 seconds from turning on Toggle Keys (beeps on Caps/Num/Scroll Lock).",
  [R(r"HKCU\Control Panel\Accessibility\ToggleKeys", "Flags", "58", "sz", d="62")],
  "The shortcut triggers by accident.", "No surprise pop-up.", "The shortcut can't turn Toggle Keys on (Settings still can).",
  "Gamers.", "signout", "low", OC)
T("usb", "keyboard_repeat_fast", "SHORTEST KEY-REPEAT DELAY",
  "Sets the keyboard 'Repeat delay' to the shortest step (about 250 ms instead of the default 500 ms).",
  [R(r"HKCU\Control Panel\Keyboard", "KeyboardDelay", "0", "sz", d="1")],
  "Held keys start repeating sooner.", "Faster key repeat in editors and some games.",
  "Accidental repeated characters if you hold keys a bit long.", "Fast typists and gamers.", "signout", "low", EX)
T("usb", "ink_workspace_off", "DISABLE WINDOWS INK WORKSPACE",
  "Policy that turns off the pen workspace/pen menu.",
  [R(POL + r"\WindowsInkWorkspace", "AllowWindowsInkWorkspace", 0)],
  "Removes a pen feature on PCs without a pen.", "No pen menu or its taskbar button.",
  "Pen shortcuts from the pen menu stop; pen input itself still works.", "PCs without a pen.", "signout", "low", EX)

# =========================== WINDOWS RESPONSIVENESS =========================
T("responsiveness", "mmcss_reserve_10", "LOWER MMCSS BACKGROUND RESERVE TO 10 %",
  "The Multimedia Class Scheduler reserves a share of CPU for low-priority work while multimedia threads (audio, games registered with MMCSS) run. The default is 20 %; this sets the documented minimum of 10 %.",
  [R(r"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile", "SystemResponsiveness", 10, d=20)],
  "Multimedia threads registered with MMCSS can use more CPU.", "Up to 90 % instead of 80 % CPU for those threads.",
  "Background tasks get less CPU while media plays.", "Gamers, audio work.", "reboot", "low", OC,
  note="Evidence of in-game gains is anecdotal.")
T("responsiveness", "drag_outline_only", "SHOW ONLY AN OUTLINE WHILE DRAGGING WINDOWS",
  "Unticks 'Show window contents while dragging'.",
  [R(r"HKCU\Control Panel\Desktop", "DragFullWindows", "0", "sz", d="1")],
  "Dragging large windows redraws them continuously.", "Smoother dragging on weak GPUs and remote sessions.",
  "You see a frame instead of the window while moving it.", "Low-end PCs, remote desktop users.", "signout", "low", OC)
T("responsiveness", "icon_label_shadow_off", "NO DROP SHADOWS ON DESKTOP ICON LABELS",
  "Removes the drop shadow behind desktop icon labels.",
  [R(ADV, "ListviewShadow", 0, d=1)],
  "One less visual effect.", "Marginally less drawing on the desktop.",
  "Labels can be harder to read on busy wallpapers.", "Low-end PCs.", "explorer", "low", OC)
T("responsiveness", "hung_app_timeout_2s", "DETECT 'NOT RESPONDING' APPS SOONER",
  "Windows marks a window as 'Not responding' after 2 seconds without response instead of 5.",
  [R(r"HKCU\Control Panel\Desktop", "HungAppTimeout", "2000", "sz")],
  "Frozen apps can be closed sooner.", "Faster 'Not responding' detection.",
  "Apps that briefly freeze (loading large files) are labelled 'Not responding' more often.", "Everyone.", "signout", "low", EX)
T("responsiveness", "service_kill_timeout_2s", "FASTER SHUTDOWN: 2 s SERVICE TIMEOUT",
  "At shutdown Windows waits 2 seconds (default 5) for each service to stop before ending it.",
  [R(r"HKLM\SYSTEM\CurrentControlSet\Control", "WaitToKillServiceTimeout", "2000", "sz", d="5000")],
  "Shutdowns can hang on a slow service.", "Shutdown finishes sooner.",
  "A service that needs longer to save its state (databases, some VPN/backup agents) can be stopped before it finishes.",
  "Desktop users without database/backup services.", "reboot", "medium", EX)
T("responsiveness", "mouse_hover_fast", "FASTER HOVER TOOLTIPS & PREVIEWS",
  "Hover delay of 100 ms instead of 400 ms for tooltips, taskbar previews and hover-to-open menus.",
  [R(r"HKCU\Control Panel\Mouse", "MouseHoverTime", "100", "sz", d="400")],
  "Hover UI appears sooner.", "Quicker tooltips and taskbar previews.",
  "Tooltips and previews pop up more readily while moving the mouse.", "Everyone.", "signout", "low", EX)

# ========================= STARTUP & BACKGROUND APPS ========================
T("background", "startup_delay_off", "NO DELAY FOR STARTUP APPS",
  "Windows normally waits about 10 seconds after sign-in before launching startup apps; this starts them immediately.",
  [R(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize", "StartupDelayInMSec", 0)],
  "Tray apps are ready sooner after sign-in.", "Chat, sync and mouse software available right away.",
  "More programs compete with Explorer right after sign-in, so the desktop can feel busy for a moment.",
  "Fast-SSD PCs.", "signout", "low", EX)
T("background", "arso_off", "DON'T AUTO-SIGN-IN AFTER UPDATES",
  "Policy that stops Windows using your sign-in info to automatically finish setup and lock the PC after an update restart.",
  [R(r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System", "DisableAutomaticRestartSignOn", 1)],
  "Your session isn't started automatically after an update.", "Nothing runs as you until you sign in.",
  "After an update restart, apps and settings finish loading only after you sign in.", "Everyone.", "none", "low", EX)
T("background", "store_apps_background_off", "POLICY: NO STORE APPS IN THE BACKGROUND",
  "Policy 'Let Windows apps run in the background' = Force Deny, for every user. On Windows 11 the per-user switch the existing tweak uses no longer covers all apps; this policy does.",
  [R(POL + r"\Windows\AppPrivacy", "LetAppsRunInBackground", 2)],
  "Store apps otherwise run background tasks when closed.", "Less background CPU, memory and network use.",
  "Store apps (Mail, Calendar, Phone Link, some games) can't send live notifications or sync while closed.",
  "Users who don't rely on Store-app notifications.", "signout", "medium", EX)

# ============================= OPTIONAL FEATURES ============================
for fid, feat, title, desc, cost in [
    ("work_folders", "WorkFolders-Client", "REMOVE WORK FOLDERS CLIENT", "Disables the Work Folders client that syncs files with a company Windows Server.", "You can't sync with a Work Folders server."),
    ("internet_printing", "Printing-Foundation-InternetPrinting-Client", "REMOVE INTERNET PRINTING CLIENT", "Disables printing to printers over HTTP/IPP via a URL.", "IPP/URL printers can't be added; USB and normal network printers still work."),
    ("xps_services", "Printing-XPSServices-Features", "REMOVE XPS SERVICES", "Disables the XPS document writer and XPS printing services.", "The 'Microsoft XPS Document Writer' printer disappears."),
    ("tiff_ifilter", "TIFFIFilter", "REMOVE TIFF IFILTER", "Disables the filter that lets Windows Search read text inside TIFF images (OCR).", "Text in scanned TIFF files isn't searchable."),
]:
    T("features", "feature_" + fid, title, desc + " The feature's files stay on disk, so turning this OFF re-enables it without downloading anything.",
      [FE(feat)], "Removes an unused Windows component.", "One less component loaded or indexed.", cost,
      "Everyone who doesn't use it.", "reboot", "low", OC)

# =============================== GPU & GRAPHICS =============================
NVG = r"HKLM\SYSTEM\CurrentControlSet\Services\nvlddmkm"
T("gpu", "nvidia_telemetry_off", "DISABLE NVIDIA TELEMETRY",
  "Sets the NVIDIA driver's 'SendTelemetryData' flag to 0, opts out of NVIDIA Control Panel data collection and turns off three NVIDIA feature-telemetry flags (RID44231/64640/66610).",
  [R(NVG + r"\Global\Startup", "SendTelemetryData", 0, gate=NVG),
   R(r"HKLM\SOFTWARE\NVIDIA Corporation\NvControlPanel2\Client", "OptInOrOutPreference", 0, gate=r"HKLM\SOFTWARE\NVIDIA Corporation"),
   R(r"HKLM\SOFTWARE\NVIDIA Corporation\Global\FTS", "EnableRID44231", 0, gate=r"HKLM\SOFTWARE\NVIDIA Corporation"),
   R(r"HKLM\SOFTWARE\NVIDIA Corporation\Global\FTS", "EnableRID64640", 0, gate=r"HKLM\SOFTWARE\NVIDIA Corporation"),
   R(r"HKLM\SOFTWARE\NVIDIA Corporation\Global\FTS", "EnableRID66610", 0, gate=r"HKLM\SOFTWARE\NVIDIA Corporation")],
  "Reduces usage data sent to NVIDIA.", "Less data collection; no effect on performance.",
  "None known.", "NVIDIA users.", "reboot", "low", OC, needs="nvidia")
T("gpu", "amd_auto_update_off", "DISABLE AMD SOFTWARE AUTO-UPDATE CHECKS",
  "Turns off AMD Software: Adrenalin Edition's automatic update check.",
  [R(r"HKCU\Software\AMD\CN", "AutoUpdate", 0, gate=r"HKCU\Software\AMD\CN")],
  "Stops update pop-ups in AMD Software.", "No background update checks.",
  "You must check for driver updates yourself.", "AMD users who update manually.", "app", "low", OC, needs="amd")

# =============================== WINDOWS (Explorer) =========================
T("windows", "explorer_open_this_pc", "OPEN FILE EXPLORER TO THIS PC",
  "File Explorer opens to 'This PC' (drives) instead of Home.",
  [R(ADV, "LaunchTo", 1)], "Drives are one click closer.", "Faster to reach your drives.",
  "Home's recent files and favourites are one click further away.", "Everyone.", "none", "low", OC)
T("windows", "copy_details_always", "ALWAYS SHOW FILE-COPY DETAILS",
  "File copy/move dialogs always open in 'More details' mode with the speed graph.",
  [R(r"HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\OperationStatusManager", "EnthusiastMode", 1)],
  "See transfer speed and remaining items without clicking.", "More information during copies.", "None.", "Everyone.", "none", "low", OC)


# ---- checks -----------------------------------------------------------------
def targets(t):
    for op in t["ops"]:
        if op["t"] == "reg":
            yield ("reg", op["p"].lower(), op["n"].lower())
        elif op["t"] == "svc":
            yield ("svc", op["n"].lower())
        elif op["t"] == "task":
            yield ("task", (op["p"] + op["n"]).lower())
        elif op["t"] == "pwr":
            yield ("pwr", op["set"].lower())
        elif op["t"] == "keys":
            yield ("keys", op["base"].lower(), op["n"].lower())
        else:
            yield (op["t"], op["n"].lower())


def main():
    here = os.path.dirname(os.path.abspath(__file__))
    old = json.load(open(os.path.join(here, "..", "data", "tweaks.json"), encoding="utf-8"))
    ids = set()
    seen = {}
    problems = []
    for t in TWEAKS:
        key = (t["c"], t["id"])
        if key in ids or any(o["c"] == t["c"] and o["id"] == t["id"] for o in old):
            problems.append(f"duplicate id {key}")
        ids.add(key)
        for tg in targets(t):
            if tg in seen:
                problems.append(f"{t['id']} and {seen[tg]} both change {tg}")
            seen[tg] = t["id"]
    # against the existing script tweaks: same value name under the same key tail, or same service/task/setting
    scripts = [(o["id"], (o.get("on") or "").lower()) for o in old]
    for t in TWEAKS:
        for tg in targets(t):
            for oid, s in scripts:
                if tg[0] in ("reg", "keys"):
                    # full key path (minus hive spelling) + value name; also the value name alone under the same key tail
                    path = tg[1].split("\\", 1)[1]
                    hive = tg[1].split("\\", 1)[0]
                    hit = re.search(re.escape(hive) + r"[^\\\"]*\\" + re.escape(path) + r"\"?\s+/v\s+\"?" + re.escape(tg[2]) + r"\"?\s", s)
                elif tg[0] == "svc":
                    hit = re.search(r"(sc config \"?|services\\)" + re.escape(tg[1]) + r"[\"\s\\]", s)
                else:
                    hit = tg[1] in s
                if hit:
                    problems.append(f"{t['id']} overlaps existing tweak {oid}: {tg}")
    for p in problems:
        print("PROBLEM:", p)
    out = os.path.join(here, "..", "data", "ops.json")
    with open(out, "w", encoding="utf-8", newline="\n") as f:
        json.dump(TWEAKS, f, indent=1, ensure_ascii=False)
    by = {}
    for t in TWEAKS:
        by[t["c"]] = by.get(t["c"], 0) + 1
    print(f"wrote {len(TWEAKS)} tweaks: {by}")
    sys.exit(1 if problems else 0)


if __name__ == "__main__":
    main()
