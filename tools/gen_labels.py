"""Builds data/labels.json: for every tweak, a clear name and what ON / OFF mean for it.

  name   display name; always says what turning the switch ON does ("Disable Print Spooler")
  on     what ON does                       ("Disables the Print Spooler service")
  state  the Windows setting while applied  ("Print Spooler service disabled")
  subj   what OFF puts back                 ("start type of the Print Spooler service")
  off    only for script tweaks: what their fixed undo command does (they have no saved original)

Run: python -I tools/gen_labels.py   (fails if a tweak has no label)
"""
import json, os, re, sys

HERE = os.path.dirname(os.path.abspath(__file__))
DATA = os.path.join(HERE, "..", "data")

# words that keep a fixed spelling; everything else is lower-cased (sentence case)
FIXED = {w.upper(): w for w in """AI AMD BSOD CEIP CMD CPUs DMA DNS DNS-over-HTTPS DVR DWM ECN EPP FX GPU HAGS HPET HTTPS ID IEEE
IPv4 IPv6 LLMNR LSA MMCSS MSI multi-GPU NetBIOS NFC/SE NIC NTFS OOBE PC PCI QoS RAM RDP RSC RSS S0 S3 SMB SQM SSDP TAPI TCP
TCP/IP TDR TPM UAC UPnP USB UTC VBS VPN VRR VSync WAP WIA XPS mDNS Windows Microsoft Xbox Cortana Bing Copilot Edge Brave
Defender OneDrive Teams Widgets Explorer Store Spotlight Recall Notepad PowerShell Outlook Cloudflare NVIDIA Intel AllJoyn
BranchCache SysMain Superfetch DiagTrack Teredo Bluetooth Ethernet FireWire Thumbs.db TIFF Start Leo Wi-Fi SmartScreen
Hyper-V BitLocker Aero LocalSystem Rewards Wallet Tor IFilter SecHealth Autochk Win32 Cast UTF-8""".split()}
FIXED.update({"WIFI": "Wi-Fi", "WIN11": "Windows 11", "ACTIONCENTER": "Action Center", "CSTATES": "C-states", "NAGLES": "Nagle's",
              "STOREAUTO": "Store auto-updates", "HIBERBOOT": "Fast Startup (hiberboot)", "IFILTER": "iFilter", "PWRTHROTTLE": "power throttling",
              "AUTOTUNINGLEVEL": "TCP auto-tuning", "MAXSTATE": "maximum processor state", "MINSTATE": "minimum processor state",
              "SECHEALTH": "Security Health service", "VIRT": "virtualization", "AUTOCHK": "Autochk", "DON'T": "don't", "S": "s",
              "USER'S": "user's", "MULTI-HOMED": "multi-homed", "OVER": "over", "WAKE-ON-LAN": "Wake-on-LAN",
              "PEEK": "Peek", "SENSE": "Sense", "MEET": "Meet", "NOW": "Now"})


def word(w, first):
    up = w.upper()
    if up in FIXED:
        return FIXED[up]
    # keep quoted menu entries and numbers as written
    if any(c.isdigit() for c in w):
        return w.lower() if w.isupper() and not first else w
    low = w.lower()
    return low[:1].upper() + low[1:] if first else low


def sentence(s):
    parts = re.split(r"(\s+)", s.strip())
    out, first = [], True
    for p in parts:
        if p.isspace() or not p:
            out.append(p)
            continue
        q = p.strip("\"'()")
        lead, trail = p[: p.find(q)] if q else "", p[p.find(q) + len(q):] if q else ""
        out.append(lead + word(q, first) + trail)
        first = False
    return "".join(out)


def quoted_title(s):
    """'"COPY AS PATH"' -> '"Copy as path"' (menu entry names keep a leading capital)."""
    return re.sub(r'"([^"]+)"', lambda m: '"' + sentence(m.group(1)) + '"', s)


def cap(s):
    return s[:1].upper() + s[1:]


VERBS = {  # name verb -> (on verb, state participle)
    "DISABLE": ("Disables", "disabled"), "ENABLE": ("Enables", "enabled"), "REMOVE": ("Removes", "removed"),
    "HIDE": ("Hides", "hidden"), "SHOW": ("Shows", "shown"), "BLOCK": ("Blocks", "blocked"), "STOP": ("Stops", "stopped"),
}


def from_name(t):
    n = t["name"]
    if t["c"] == "services":
        svc = " ".join(w.upper() if w.strip("()").upper() in FIXED and FIXED[w.strip("()").upper()].isupper() else
                       (w.replace(w.strip("()"), FIXED[w.strip("()").upper()]) if w.strip("()").upper() in FIXED else w)
                       for w in n.title().split())
        svc = svc.replace(" And ", " and ").replace(" Of ", " of ")
        kind = "" if svc.lower().endswith(("service", "driver", "controller")) else " service"
        return {"name": f"Disable {svc}", "on": f"Sets {svc}{kind} to Disabled", "state": f"{svc}{kind} disabled",
                "subj": f"start type of {svc}{kind}"}
    if t["c"] == "tasks":
        task = sentence(n)
        low = task if task.split()[0].upper() in FIXED else task[:1].lower() + task[1:]
        name = f"Disable the {low}" if low.lower().endswith("tasks") else f"Disable the {low} task"
        return {"name": name, "on": f"Disables the {low} scheduled task" + ("s" if low.lower().endswith("tasks") else ""),
                "state": cap(low) + (" disabled" if low.lower().endswith("tasks") else " task disabled"), "subj": "state of the task (enabled or disabled)"}
    m = re.match(r"DISABLE (.+) ACCESS$", n)
    if t["c"] == "privacy" and m:
        o = sentence(m.group(1))
        o = o[:1].lower() + o[1:] if o.split()[0].upper() not in FIXED else o
        return {"name": f"Block app access to {o}", "on": f"Stops apps using your {o}", "state": f"App access to {o} blocked", "subj": f"{o} permission"}
    m = re.match(r"(DISABLE|ENABLE|REMOVE|HIDE|SHOW|BLOCK|STOP) (.+)", n)
    if m:
        verb, obj = m.group(1), m.group(2)
        on_v, part = VERBS[verb]
        o = quoted_title(sentence(obj))
        if t["c"] == "contextmenu" and obj.startswith('"'):
            return {"name": f"Remove {o} from the right-click menu" if verb == "REMOVE" else f"{verb.title()} {o}",
                    "on": f"{on_v} {o} from the right-click menu", "state": f"{o} not in the right-click menu", "subj": f"{o} menu entry"}
        o_first = o[:1].lower() + o[1:] if o[:1].isupper() and o.split()[0].upper() not in FIXED else o
        return {"name": f"{verb.title()} {o_first}", "on": f"{on_v} {o_first}", "state": f"{cap(re.sub(r'^the ', '', o_first))} {part}", "subj": f"{o_first} settings"}
    m = re.match(r'ADD (".+")', n)
    if m:
        o = quoted_title(m.group(1))
        return {"name": f"Add {o} to the right-click menu", "on": f"Adds {o} to the right-click menu", "state": f"{o} in the right-click menu",
                "subj": "right-click menu (the entry is removed again)"}
    return None


# hand-written labels where a rule can't say it well (key -> fields)
O = {}


def L(key, name, on, state, subj, off=None):
    O[key] = {"name": name, "on": on, "state": state, "subj": subj, **({"off": off} if off else {})}


# ---- script tweaks: no saved original, OFF runs a fixed undo command ----
SCRIPT = "Runs the undo command: "
L("windows|disable_firewall", "Disable Windows Firewall", "Turns Windows Firewall off for every network profile", "Firewall off", "firewall",
  SCRIPT + "turns Windows Firewall back on for every profile")
L("gpu|disable_firewall", "Disable Windows Firewall", "Turns Windows Firewall off for every network profile", "Firewall off", "firewall",
  SCRIPT + "turns Windows Firewall back on for every profile")
L("windows|disable_hibernate", "Disable hibernation", "Turns hibernation off and deletes hiberfil.sys", "Hibernation off", "hibernation",
  SCRIPT + "turns hibernation back on")
L("power|disable_hibernate", "Disable hibernation", "Turns hibernation off and deletes hiberfil.sys", "Hibernation off", "hibernation",
  SCRIPT + "turns hibernation back on")
L("windows|disable_sleep", "Disable sleep", "Sets sleep to Never and turns hibernation off", "Sleep and hibernation off", "sleep",
  SCRIPT + "sets the Windows sleep timers back to their usual values and turns hibernation on")
L("power|disable_sleep", "Disable sleep", "Sets sleep to Never and turns hibernation off", "Sleep and hibernation off", "sleep",
  SCRIPT + "sets the Windows sleep timers back to their usual values and turns hibernation on")
L("windows|high_performance", "Switch to the High performance plan", "Makes High performance the active power plan", "High performance plan active", "power plan",
  SCRIPT + "makes Balanced the active power plan")
L("windows|disable_cstates", "Disable CPU core parking (High performance plan)", "Keeps all cores unparked in the High performance plan", "Core parking off in that plan", "core parking",
  SCRIPT + "writes the usual core-parking values to the High performance plan")
L("windows|remove_teams", "Remove Microsoft Teams", "Uninstalls the Microsoft Teams app", "Teams app removed", "Teams",
  "Can't be undone automatically: reinstall Teams from the Microsoft Store")
L("windows|remove_phone_link", "Remove Phone Link", "Uninstalls the Phone Link app", "Phone Link removed", "Phone Link",
  "Can't be undone automatically: reinstall Phone Link from the Microsoft Store")
L("windows|remove_dev_home", "Remove Dev Home", "Uninstalls the Dev Home app", "Dev Home removed", "Dev Home",
  "Can't be undone automatically: reinstall Dev Home from the Microsoft Store")
L("gaming|timer_resolution", "Use the platform tick timer", "Sets the boot option useplatformtick to yes", "Platform tick on (after restart)", "boot timer option",
  SCRIPT + "removes the useplatformtick boot option")
L("gaming|optimize_games", "Disable the dynamic tick", "Sets the boot option disabledynamictick to yes", "Dynamic tick off (after restart)", "boot timer option",
  SCRIPT + "removes the disabledynamictick boot option")
L("gaming|disable_hpet", "Don't force the HPET timer", "Sets the boot option useplatformclock to false", "HPET not forced (after restart)", "boot timer option",
  SCRIPT + "removes the useplatformclock boot option")
L("gaming|msi_mode", "Enable MSI interrupts for GPU and network", "Turns on message-signaled interrupts for display and network devices that support them", "MSI mode on (after restart)", "interrupt mode",
  SCRIPT + "turns MSI mode off again for those devices")
L("power|ultimate_power", "Add and use the Ultimate Performance plan", "Creates the Ultimate Performance plan if needed and makes it active", "Ultimate Performance plan active", "power plan",
  SCRIPT + "makes Balanced the active power plan (the Ultimate plan stays in the list)")
L("power|disable_hybrid_sleep", "Disable hybrid sleep", "Turns hybrid sleep and hibernation off", "Hybrid sleep off", "hybrid sleep",
  SCRIPT + "turns hibernation and hybrid sleep back on")
L("power|disable_hiberboot", "Disable Fast Startup", "Turns Fast Startup (hiberboot) and hibernation off", "Fast Startup off", "Fast Startup",
  SCRIPT + "turns Fast Startup and hibernation back on")
L("network|disable_teredo", "Disable Teredo", "Turns the Teredo IPv6 tunnel off", "Teredo off", "Teredo", SCRIPT + "sets Teredo back to its default state")
L("network|disable_nagle", "Disable Nagle's algorithm", "Sends small TCP packets at once on every network interface", "Nagle's algorithm off", "TCP acknowledgement settings",
  SCRIPT + "removes the TcpAckFrequency and TCPNoDelay values again")
L("network|disable_rss", "Disable receive-side scaling", "Turns RSS off in the TCP stack", "RSS off", "RSS", SCRIPT + "turns RSS back on")
L("network|disable_rsc", "Disable receive segment coalescing", "Turns RSC off in the TCP stack", "RSC off", "RSC", SCRIPT + "turns RSC back on")
L("network|disable_autotuninglevel", "Disable TCP auto-tuning", "Sets the TCP receive window auto-tuning level to disabled", "Auto-tuning off", "auto-tuning",
  SCRIPT + "sets auto-tuning back to normal")
L("network|use_cloudflare_dns", "Use Cloudflare DNS", "Sets 1.1.1.1 and 1.0.0.1 as the DNS servers of every physical adapter", "DNS: 1.1.1.1 / 1.0.0.1", "DNS servers",
  SCRIPT + "sets the adapters back to automatic DNS (from your router); a DNS address you had typed in yourself is not brought back")
L("network|disable_ecn", "Disable ECN", "Turns Explicit Congestion Notification off", "ECN off", "ECN", SCRIPT + "sets ECN back to its default")
L("network|disable_timestamps", "Disable TCP timestamps", "Turns TCP timestamps off", "TCP timestamps off", "TCP timestamps", SCRIPT + "turns TCP timestamps back on")
L("personalize|small_icons", "Use small taskbar icons", "Uses small taskbar buttons and restarts Explorer", "Small taskbar icons", "taskbar size",
  SCRIPT + "sets normal taskbar icons and restarts Explorer")
for c in ["blue", "teal", "green", "lime", "gold", "orange", "red", "pink", "purple", "gray"]:
    nm = "graphite" if c == "gray" else ("Windows blue" if c == "blue" else c)
    L(f"personalize|accent_{c}", f"Accent colour: {nm}", f"Sets the Windows accent colour to {nm} and restarts Explorer", f"Accent colour {nm}", "accent colour",
      SCRIPT + "sets the Windows default accent colour and restarts Explorer (your own earlier colour is not saved)")

# ---- original tweaks whose short names were vague or pointed the wrong way ----
def L2(key, name, on, state, subj, desc=None):
    O[key] = {"name": name, "on": on, "state": state, "subj": subj, **({"desc": desc} if desc else {})}


W, G, P, PW = "windows|", "gaming|", "gpu|", "power|"
L2(W + "disable_actioncenter", "Disable Action Center", "Turns off the Action Center notification panel", "Action Center off", "Action Center setting")
L2(W + "disable_background", "Block background apps", "Stops Store apps running in the background", "Background apps blocked", "background apps setting")
L2(W + "disable_extensions", "Hide file extensions", "Hides file name extensions (.txt, .exe) in File Explorer", "File extensions hidden", "file extension setting")
L2(W + "disable_hidden", "Don't show hidden files", "Hides hidden and system files in File Explorer", "Hidden files not shown", "hidden files setting")
L2(W + "disable_remote", "Disable Remote Desktop", "Turns off Remote Desktop connections to this PC", "Remote Desktop off", "Remote Desktop setting")
L2(W + "disable_restore", "Disable System Restore", "Turns off System Restore", "System Restore off", "System Restore setting")
L2(W + "disable_search", "Disable Windows Search indexing", "Disables and stops the Windows Search service", "Search indexing off", "start type of the Windows Search service (and starts it again)")
L2(W + "disable_storeauto", "Disable Store auto-updates", "Stops the Microsoft Store updating apps by itself", "Store auto-updates off", "Store update setting")
L2(W + "disable_xbox", "Disable Xbox services", "Disables the Xbox background services", "Xbox services disabled", "Xbox service settings")
L2(W + "disable_core_parking", "Disable CPU core parking", "Keeps every CPU core unparked in the active power plan", "Core parking off", "core parking setting")
L2(W + "disable_boost", "Disable CPU turbo boost", "Turns off processor boost in the active power plan", "Turbo boost off", "boost mode setting")
L2(W + "disable_cooling", "Use active CPU cooling", "Sets the cooling policy to Active: fans speed up before the CPU slows down", "Cooling policy: active", "cooling policy",
   "Sets the processor cooling policy of the active plan to Active (fans first, then slowing the CPU).")
L2(W + "disable_hetero", "Disable heterogeneous CPU scheduling", "Turns off hybrid-core aware scheduling", "Hetero scheduling off", "scheduling setting")
L2(W + "disable_idle", "Disable CPU idle states", "Stops the CPU entering idle states in the active power plan", "CPU idle states off", "processor idle setting")
L2(W + "disable_maxstate", "Maximum processor state 100 %", "Sets the maximum processor state of the active plan to 100 %", "Max processor state 100 %", "maximum processor state")
L2(W + "disable_minstate", "Minimum processor state 100 %", "Keeps the CPU at full speed (minimum processor state 100 %)", "Min processor state 100 %", "minimum processor state")
L2(W + "disable_mitigations", "Disable Spectre and Meltdown mitigations", "Turns off the CPU security mitigations for Spectre and Meltdown", "Mitigations off", "mitigation settings")
L2(W + "disable_pwrthrottle", "Disable power throttling", "Stops Windows slowing down background apps to save power", "Power throttling off", "power throttling setting")
L2(W + "disable_virt", "Disable virtualization-based security", "Turns off virtualization-based security", "VBS off", "VBS setting")
L2(W + "disable_sechealth", "Disable the Security Health service", "Disables and stops the Windows Security Health service", "Security Health service off", "start type of the Security Health service (and starts it again)")
L2(W + "disable_speech", "Disable online speech recognition", "Turns off online speech recognition and its data collection", "Online speech off", "speech privacy setting")
L2(W + "disable_widgets_win11", "Disable widgets", "Turns off Windows 11 widgets by policy", "Widgets off", "widgets policy")
L2(W + "disable_edge_bg_win11", "Stop Edge running in the background", "Stops Microsoft Edge keeping processes running after you close it", "Edge background mode off", "Edge background policy")
L2(W + "disable_onedrive_win11", "Disable OneDrive sync", "Turns off OneDrive file sync by policy", "OneDrive sync off", "OneDrive policy")
L2(W + "disable_stickers", "Disable desktop stickers", "Turns off the desktop stickers feature", "Desktop stickers off", "stickers setting")
L2(W + "disable_tabs_explorer", "Disable File Explorer tabs", "Opens folders in separate windows instead of tabs", "Explorer tabs off", "Explorer tabs setting")
L2(W + "remove_chat", "Hide the Chat icon", "Removes the Teams Chat icon from the taskbar", "Chat icon hidden", "taskbar Chat setting")
L2(W + "disable_co_creator", "Disable Paint Cocreator", "Turns off Cocreator and the AI features in Paint", "Paint AI features off", "Paint AI setting")
L2(W + "enable_compact_mode", "Use compact view in File Explorer", "Uses compact spacing between items in File Explorer", "Compact view on", "Explorer spacing setting")
L2(W + "disable_start_recommended", "Hide Recommended in Start", "Hides the Recommended section of the Start menu", "Recommended hidden", "Start Recommended setting")
L2(W + "disable_combo_anim", "Disable drop-down list animation", "Turns off the slide animation of drop-down lists", "Drop-down animation off", "drop-down animation setting")
L2(W + "disable_listbox_smooth", "Disable smooth scrolling in lists", "Makes lists jump instead of scrolling smoothly", "Smooth list scrolling off", "list scrolling setting")
L2(W + "disable_font_smoothing", "Disable ClearType font smoothing", "Turns off font smoothing (text looks sharper but jagged)", "Font smoothing off", "font smoothing setting")
L2(W + "enable_end_task", "Add End task to the taskbar menu", "Adds End task to the right-click menu of taskbar buttons", "End task in taskbar menu", "taskbar End task setting")
L2(W + "disable_folder_discovery", "Disable automatic folder type discovery", "Stops File Explorer changing a folder's view based on its contents", "Folder discovery off", "folder view settings")
L2(W + "disable_home_gallery", "Hide Home and Gallery in File Explorer", "Removes Home and Gallery from the File Explorer navigation pane", "Home and Gallery hidden", "Explorer navigation setting")
L2(W + "enable_classic_menu", "Use the classic right-click menu", "Brings back the full Windows 10 style right-click menu on Windows 11", "Classic right-click menu", "right-click menu style")
L2(W + "enable_start_layout", "Use the previous Start layout", "Brings back the earlier Windows 11 Start menu layout", "Previous Start layout", "Start layout setting")
L2(W + "disable_start_bing", "Remove Bing results from Start search", "Stops Start search showing web results from Bing", "No Bing results in Start", "Start search setting")
L2(W + "enable_dark_theme", "Use dark mode", "Switches apps and Windows to dark mode", "Dark mode on", "app and system colour mode")
L2(W + "enable_numlock", "Turn on Num Lock at startup", "Switches Num Lock on when Windows starts", "Num Lock on at startup", "Num Lock setting")
L2(W + "enable_scrollbars", "Always show scrollbars", "Keeps scrollbars visible instead of hiding them", "Scrollbars always visible", "scrollbar setting")
L2(W + "enable_battery_pct", "Show battery percentage", "Shows the battery percentage next to the battery icon", "Battery percentage shown", "battery icon setting")
L2(G + "game_mode", "Enable Game Mode", "Turns on Windows Game Mode", "Game Mode on", "Game Mode setting")
L2(G + "hags", "Enable hardware-accelerated GPU scheduling", "Turns on hardware-accelerated GPU scheduling (restart needed)", "GPU scheduling on", "GPU scheduling setting")
L2(G + "disable_game_dvr", "Disable Game DVR", "Turns off Game DVR background recording", "Background recording off", "recording settings")
L2(G + "disable_game_bar", "Disable Xbox Game Bar", "Turns off the Xbox Game Bar overlay and its shortcuts", "Game Bar off", "Game Bar settings")
L2(G + "disable_memory_integrity", "Disable Memory Integrity", "Turns off Core isolation Memory Integrity (restart needed)", "Memory Integrity off", "Memory Integrity setting")
L2(G + "disable_vbs", "Disable virtualization-based security (VBS)", "Turns off virtualization-based security (restart needed)", "VBS off", "VBS setting")
L2(G + "enable_vrr", "Enable variable refresh rate", "Turns on variable refresh rate for supported displays", "VRR on", "VRR setting")
L2(G + "disable_spectre_meltdown", "Disable Spectre and Meltdown mitigations", "Turns off the CPU security mitigations for Spectre and Meltdown", "Mitigations off", "mitigation settings")
L2(G + "disable_tpm", "Disable Credential Guard", "Turns off Credential Guard and related TPM-based protection", "Credential Guard off", "Credential Guard setting")
L2(P + "disable_gpu_timeout", "Lengthen the GPU timeout (TdrDelay)", "Waits 8 s instead of 2 s before Windows resets a stuck graphics driver", "GPU timeout 8 s", "GPU timeout")
L2(P + "enable_gpu_accel", "Force GPU acceleration for windows", "Makes window rendering always use the GPU", "GPU acceleration forced", "hardware acceleration setting")
L2(P + "disable_tray_notifications", "Disable the notification center", "Turns off the notification center and calendar flyout", "Notification center off", "notification center setting")
L2(P + "enable_bsod_verbose", "Show details on blue screens", "Shows technical details on blue screens instead of the sad face", "Detailed blue screens", "blue screen setting")
L2(P + "enable_cross_device", "Enable clipboard history and cross-device resume", "Turns on clipboard history and resuming activities on other devices", "Clipboard history on", "clipboard settings")
L2(P + "disable_logon_blur", "No blur on the sign-in screen", "Shows the sign-in background without the blur effect", "Sign-in blur off", "sign-in blur setting")
L2(P + "enable_logon_verbose", "Show detailed startup and shutdown messages", "Shows what Windows is doing during startup, sign-in and shutdown", "Verbose status messages", "status message setting")
L2(P + "disable_outlook_new", "Block the new Outlook", "Stops the new Outlook app replacing classic Outlook", "New Outlook blocked", "Outlook setting")
L2(P + "disable_multiplane_overlay", "Disable multiplane overlay (MPO)", "Turns off MPO, which can fix flicker and stutter in some games", "MPO off", "MPO setting")
L2(P + "disable_s0_network", "No network during Modern Standby", "Disconnects the network while the PC is in Modern Standby", "Network off in standby", "standby network setting")
L2(P + "enable_s3_sleep", "Use S3 sleep instead of Modern Standby", "Switches the PC to classic S3 sleep (restart needed)", "S3 sleep", "sleep mode setting")
L2(P + "disable_settings_home", "Hide the Settings home page", "Removes the Home page from the Settings app", "Settings home hidden", "Settings home setting")
L2(P + "disable_rdp_warnings", "Disable Remote Desktop unsigned-file warnings", "Stops warnings when opening unsigned Remote Desktop files", "RDP warnings off", "RDP warning setting")
L2(PW + "disk_timeout", "Never turn off hard disks", "Sets the hard-disk sleep timer to Never (plugged in)", "Disks never sleep", "hard-disk timeout")
L2(PW + "display_timeout", "Never turn off the display", "Sets the screen-off timer to Never (plugged in)", "Display never turns off", "display timeout")
L2(PW + "disable_wifi_sense", "Disable Wi-Fi Sense", "Turns off Wi-Fi Sense hotspot sharing", "Wi-Fi Sense off", "Wi-Fi Sense setting")
L2(PW + "disable_connected_standby", "Disable Modern Standby", "Turns off Modern Standby so S3 sleep can be used (restart needed)", "Modern Standby off", "standby setting")
L2(PW + "disable_throttle_state", "Disable processor throttle states", "Stops Windows lowering CPU speed with throttle states", "Throttle states off", "throttle setting")
L2(PW + "disable_usb_power_save", "Disable USB hub power saving", "Stops USB root hubs being powered down to save energy", "USB hub power saving off", "USB hub power setting")
L2(PW + "disable_store_auto", "Disable Store auto-updates", "Stops the Microsoft Store updating apps by itself", "Store auto-updates off", "Store update setting")
L2("network|enable_tcp_nodelay", "Enable TCP no-delay", "Sends TCP packets immediately without waiting to combine them", "TCP no-delay on", "TCP delay setting")
L2("network|disable_qos", "Remove the QoS bandwidth reserve", "Sets the reservable QoS bandwidth to 0 %", "QoS reserve 0 %", "QoS bandwidth policy")
L2("privacy|privacy_screenshots", "Block borderless screenshots by apps", "Stops apps taking screenshots without the yellow border", "Borderless screenshots blocked", "screenshot permission")
L2("privacy|privacy_background_apps", "Block background apps", "Stops apps running in the background", "Background apps blocked", "background apps permission")
L2("contextmenu|new_bing_search", "Turn off web results in Windows search", "Stops the search box showing web suggestions", "No web results in search", "search box setting")

# engine tweaks a rule can't phrase well are written by hand in tools/labels_extra.json


# Risk of the original tweaks (the newer ones carry it in their details). Shown as a badge.
RISK = {
    "high": ["windows|disable_defender", "gpu|disable_defender", "windows|disable_firewall", "gpu|disable_firewall",
             "windows|disable_mitigations", "gaming|disable_spectre_meltdown", "windows|disable_virt", "gaming|disable_vbs",
             "gaming|disable_memory_integrity", "gaming|disable_tpm", "windows|disable_sechealth"],
    "medium": ["windows|disable_restore", "windows|remove_teams", "windows|remove_phone_link", "windows|remove_dev_home",
               "windows|disable_idle", "windows|disable_cstates", "gaming|timer_resolution", "gaming|optimize_games",
               "gaming|disable_hpet", "gaming|msi_mode", "network|disable_ipv6", "windows|disable_remote", "gpu|disable_remote_assist",
               "gpu|enable_s3_sleep", "power|disable_connected_standby", "gpu|disable_rdp_warnings",
               "network|disable_autotuninglevel", "network|disable_rss", "windows|disable_font_smoothing"],
}


def build():
    tweaks = json.load(open(os.path.join(DATA, "tweaks.json"), encoding="utf-8")) + json.load(open(os.path.join(DATA, "ops.json"), encoding="utf-8"))
    out, missing = {}, []
    extra = json.load(open(os.path.join(HERE, "labels_extra.json"), encoding="utf-8"))
    for t in tweaks:
        key = f"{t['c']}|{t['id']}"
        lab = O.get(key) or extra.get(key) or from_name(t)
        if not lab:
            missing.append(f"{key}  {t['name']}")
            continue
        if not t.get("ops") and "off" not in lab:
            missing.append(f"{key}  (script tweak without an OFF text)")
        lab = dict(lab)
        for level, keys in RISK.items():
            if key in keys:
                lab["risk"] = level
        out[key] = lab
    unknown = [k for ks in RISK.values() for k in ks if k not in out]
    if unknown:
        missing.append("RISK lists unknown tweaks: " + ", ".join(unknown))
    if missing:
        print("no label for:\n  " + "\n  ".join(missing))
        return 1
    json.dump(out, open(os.path.join(DATA, "labels.json"), "w", encoding="utf-8", newline="\n"), indent=1, ensure_ascii=False)
    print(f"wrote {len(out)} labels")
    return 0


if __name__ == "__main__":
    sys.exit(build())
