# Heavy Tweaks

*A Cheetix preset · for: Aggressive optimization*

Everything in Moderate, plus deeper service and feature changes: location, Game Bar captures, SysMain, network extras and some optional Windows features. Some Windows features stop working.

**148 changes in total.** This preset includes all 95 changes of [Moderate Tweaks](moderate.md) and adds the 53 below.

Nothing is applied until you review the list in Cheetix and press **Apply**. Changes that are already on are left alone, and changes your version of Windows doesn't support are skipped.

## What it changes

### Privacy & Telemetry

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable location for all users** | Turns off the device-wide location switch (the top switch in Settings > Privacy > Location), so no account or app on this PC can read your position. The existing Location tweak only affects your own account's apps. | No location lookups (Wi-Fi/IP based or GPS) by any app. | Maps, weather, 'Find my device', automatic time zone and any app that needs your location stop getting it. | Right away |

### Windows Services

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable Geolocation Service** | Provides the PC's location to apps. This tweak sets the 'lfsvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | No app gets your location; Find my device and automatic time zone stop working. | After a restart |
| **Disable Auto Time Zone Updater** | Sets the time zone automatically from location. This tweak sets the 'tzautoupdate' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Time zone must be set by hand when travelling. | After a restart |

### Network

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable energy-efficient Ethernet** | Sets the 'Energy-Efficient Ethernet' (802.3az) advanced property to Disabled on every wired adapter whose driver exposes it. | Can remove small latency spikes and occasional link drops some switch/driver combinations show with EEE. | Slightly higher power use (well under a watt per link). Adapters without the setting are left alone. | Right away (the network adapter restarts) |
| **Disable Wake-on-LAN** | Disables 'Wake on Magic Packet' and 'Wake on Pattern Match' on wired adapters that expose them. | No unexpected wake-ups caused by network activity. | The PC can't be woken remotely (WoL tools, some NAS or backup schedules). | Right away (the network adapter restarts) |
| **Disable NIC vendor power saving** | Turns off driver-specific power-saving features on adapters that expose them: Realtek 'Green Ethernet', 'Power Saving Mode', 'Gigabit Lite', 'Advanced EEE', Intel 'Auto Power Saver' and 'Ultra Low Power Mode'. Each adapter only gets the names its own driver uses. | Avoids brief disconnects or a link that comes back at a lower speed. | A little more power use; on laptops on battery that means shorter battery life. | Right away (the network adapter restarts) |
| **Don't let Windows turn off the network adapter** | Clears 'Allow the computer to turn off this device to save power' on wired adapters (PnPCapabilities = 24), the same as unticking it in Device Manager. | Avoids 'network adapter disappeared' or slow-to-reconnect problems after idle or sleep. | The adapter stays powered during sleep (slightly higher sleep power on laptops). | After a restart |
| **Shorten TCP TIME_WAIT to 30 seconds** | Closed TCP connections stay reserved for 30 s instead of the default 120 s (TcpTimedWaitDelay). | Ports are freed sooner for download managers, torrent clients and local servers. | In theory a very late packet of an old connection could be mixed up with a new one; harmless in normal use. | After a restart |

### DNS & Name Resolution

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable LLMNR** | Policy that turns off Link-Local Multicast Name Resolution, the fallback that broadcasts a name on the local network when DNS can't answer. | Closes that attack and stops broadcast name lookups. | Old devices on the LAN reachable only by LLMNR name stop resolving by name; use their IP or DNS name. | Right away |
| **Disable smart multi-homed name resolution** | Policy that stops Windows from sending each DNS query to every network adapter at once and taking the fastest answer. | Lookups use one adapter's DNS in order, keeping VPN lookups inside the VPN. | If the first adapter's DNS is slow or broken, Windows waits instead of racing another adapter. | Right away |
| **Don't cache failed DNS lookups** | Windows normally remembers a failed lookup ('name not found') for up to 15 minutes; this sets that time to 0 (MaxNegativeCacheTtl). | Sites that were briefly unreachable (after changing DNS or rebooting the router) resolve again immediately. | Names that really don't exist are asked again every time (negligible traffic). | After a restart |

### Windows Services

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable WAP Push Message Routing** | Routes WAP push messages; Windows uses it mainly for device-management (MDM/Intune) enrollment and some diagnostics messaging. This tweak sets the 'dmwappushservice' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Work/school device enrollment (Intune/MDM) stops working. | After a restart |
| **Disable Program Compatibility Assistant** | Watches programs you run and offers compatibility fixes when an old program fails or installs incorrectly. This tweak sets the 'PcaSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | No automatic compatibility help for legacy software. | After a restart |
| **Disable Game DVR & Broadcast User Service** | Per-user service Game Bar uses to record clips and broadcast. This tweak sets the 'BcastDVRUserService' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Game Bar recording (Win+Alt+R) and clips stop working. | After a restart |
| **Disable Phone Service** | Manages telephony state on the device. This tweak sets the 'PhoneSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Calls through Phone Link and some VoIP apps may stop working. | After a restart |
| **Disable Payments and NFC/SE Manager** | Manages NFC payments and secure elements. This tweak sets the 'SEMgrSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | NFC payments stop (most desktops have no NFC). | After a restart |
| **Disable Messaging Service** | Per-user service for text messaging (SMS relay). This tweak sets the 'MessagingService' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | SMS/MMS in Phone Link may stop. | After a restart |
| **Disable Contact Data Indexing** | Indexes contacts for fast search in Mail/People. This tweak sets the 'PimIndexMaintenanceSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Contact search in Mail/People may be slower or incomplete. | After a restart |
| **Disable Mobile Hotspot Service** | Lets the PC share its internet connection as a hotspot. This tweak sets the 'icssvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Mobile Hotspot can't be turned on. | After a restart |
| **Disable Link-Layer Topology Discovery Mapper** | Draws the network map. This tweak sets the 'lltdsvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | The network map feature has nothing to show. | After a restart |
| **Disable SSDP Discovery** | Discovers UPnP devices (media servers, some printers, smart TVs) on the network. This tweak sets the 'SSDPSRV' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | UPnP devices stop appearing in File Explorer > Network; casting to some DLNA TVs stops. | After a restart |
| **Disable UPnP Device Host** | Lets this PC host UPnP devices for others. This tweak sets the 'upnphost' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | This PC can't act as a UPnP/DLNA device. | After a restart |
| **Disable Function Discovery Resource Publication** | Announces this PC and its shared resources on the network. This tweak sets the 'FDResPub' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Other PCs stop seeing this PC under Network. | After a restart |
| **Disable TCP/IP NetBIOS Helper** | Supports NetBIOS over TCP/IP name resolution. This tweak sets the 'lmhosts' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | NetBIOS name resolution and old-style network browsing stop. | After a restart |
| **Disable Sensor Monitoring Service** | Monitors sensors such as the ambient-light sensor. This tweak sets the 'SensrSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Adaptive brightness from the light sensor stops. | After a restart |
| **Disable Sensor Data Service** | Delivers data from sensors to apps. This tweak sets the 'SensorDataService' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Apps can't read sensor data. | After a restart |
| **Disable Parental Controls** | Enforces Microsoft Family Safety settings. This tweak sets the 'WpcMonSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Family Safety limits and activity reports aren't enforced on this PC. | After a restart |
| **Disable Distributed Link Tracking Client** | Keeps shortcuts working when the files they point to move between NTFS volumes or network shares. This tweak sets the 'TrkWks' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Shortcuts to moved files may break instead of being repaired. | After a restart |
| **Disable Still Image Acquisition Events** | Delivers scanner button events to apps. This tweak sets the 'WiaRpc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Pressing Scan on a device no longer starts an app. | After a restart |
| **Disable Application Layer Gateway** | Plug-ins for protocols used through Internet Connection Sharing. This tweak sets the 'ALG' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Legacy protocol helpers for ICS (e.g. old FTP) stop. | After a restart |
| **Disable Certificate Propagation** | Copies certificates from inserted smart cards to your certificate store. This tweak sets the 'CertPropSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Smart-card certificates aren't added automatically. | After a restart |
| **Disable Diagnostic Execution Service** | Runs diagnostic scripts for troubleshooters. This tweak sets the 'diagsvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Troubleshooters that run scripts can't run. | After a restart |
| **Disable Recommended Troubleshooting** | Runs Microsoft's recommended troubleshooters automatically. This tweak sets the 'TroubleshootingSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | No automatic recommended troubleshooting. | After a restart |
| **Disable Intel Management Engine Local Management** | Intel AMT/vPro local management service. This tweak sets the 'LMS' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Intel AMT remote management (business PCs) stops; consumer PCs don't use it. | After a restart |
| **Disable FireWire (IEEE 1394) Controller** | Driver for FireWire controllers. This tweak sets the '1394ohci' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | FireWire devices (old cameras, audio interfaces) stop working. | After a restart |

### Scheduled Tasks

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable the family safety monitor task** | Initialises Family Safety monitoring. This tweak disables the scheduled task '\Microsoft\Windows\Shell\FamilySafetyMonitor'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | Family Safety activity monitoring stops. | Right away |
| **Disable the family safety refresh task** | Syncs Family Safety settings. This tweak disables the scheduled task '\Microsoft\Windows\Shell\FamilySafetyRefreshTask'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | Family Safety settings don't refresh. | Right away |

### Windows Debloat

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Block device companion software auto-install** | Stops driver packages from running vendor 'co-installers' - the mechanism that silently installs apps like Razer Synapse or Logitech G HUB when you plug a device in. | No surprise companion apps when plugging in devices. | Some devices may install with reduced features until you install the vendor software yourself. | Right away |
| **Skip the lock screen** | Policy that goes straight to the sign-in screen (PIN/password) instead of showing the picture lock screen first. Fully reversible: nothing is deleted. | Faster to the sign-in box. | No lock-screen picture, clock, notifications or widgets; you still have to sign in. | Right away |

### Gaming

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable Game Bar captures (your account)** | Turns off the per-user capture switch so Game Bar can't record or screenshot games. The existing 'Disable Game DVR' tweak is the all-users policy. | No capture overhead from Game Bar. | Game Bar recording, screenshots and clips stop working for your account. | Right away |
| **Allow global timer resolution requests** | Since Windows 10 2004 a program's request for a finer system timer only affects that program. On Windows 11 this value restores the old behaviour where such a request applies system-wide. | Some players report steadier frame pacing with a 0.5 ms timer; results vary by game and PC. | Higher idle power use and shorter battery life while any program requests a fine timer. No effect unless something requests one. | After a restart |

### RAM & Memory

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable SysMain (Superfetch)** | Stops the service that preloads frequently used apps into RAM and tunes memory and boot prefetching. | Less background disk and memory activity. | Frequently used apps can start slower, especially from a hard disk; boot prefetching stops. Memory compression can no longer be changed while it is off. | After a restart |
| **Disable page combining** | Stops Windows from periodically scanning RAM for identical pages to merge them. | Removes that background scan. | Slightly higher memory use. | After a restart |
| **Keep the kernel and drivers in RAM** | Prevents Windows from paging kernel-mode code and drivers out to the page file. | Avoids rare stalls after the PC has been idle. | A few hundred MB less RAM for apps; little measurable benefit on PCs with plenty of RAM. | After a restart |

### Storage & SSD

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Stop creating 8.3 short file names** | New files no longer get an extra DOS-style short name (like PROGRA~1). Existing short names stay. | Faster file creation in folders with many similar names. | Very old 16-bit/DOS-era installers and tools that need short names may fail on new files. | After a restart |
| **Give NTFS more cache memory** | Raises the paged-pool memory NTFS may use for its metadata cache (NtfsMemoryUsage = 2). | Can help with very large numbers of files (big repos, game assets). | More kernel memory use; little effect on typical use. | After a restart |

### Responsiveness

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Detect 'Not responding' apps sooner** | Windows marks a window as 'Not responding' after 2 seconds without response instead of 5. | Faster 'Not responding' detection. | Apps that briefly freeze (loading large files) are labelled 'Not responding' more often. | After signing out |
| **No drop shadows on desktop icon labels** | Removes the drop shadow behind desktop icon labels. | Marginally less drawing on the desktop. | Labels can be harder to read on busy wallpapers. | After Explorer restarts |

### Startup & Background Apps

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Don't let Store apps run in the background** | Policy 'Let Windows apps run in the background' = Force Deny, for every user. On Windows 11 the per-user switch the existing tweak uses no longer covers all apps; this policy does. | Less background CPU, memory and network use. | Store apps (Mail, Calendar, Phone Link, some games) can't send live notifications or sync while closed. | After signing out |

### Optional Features

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Remove work folders client** | Disables the Work Folders client that syncs files with a company Windows Server. The feature's files stay on disk, so turning this OFF re-enables it without downloading anything. | One less component loaded or indexed. | You can't sync with a Work Folders server. | After a restart |
| **Remove internet printing client** | Disables printing to printers over HTTP/IPP via a URL. The feature's files stay on disk, so turning this OFF re-enables it without downloading anything. | One less component loaded or indexed. | IPP/URL printers can't be added; USB and normal network printers still work. | After a restart |
| **Remove XPS services** | Disables the XPS document writer and XPS printing services. The feature's files stay on disk, so turning this OFF re-enables it without downloading anything. | One less component loaded or indexed. | The 'Microsoft XPS Document Writer' printer disappears. | After a restart |
| **Remove TIFF iFilter** | Disables the filter that lets Windows Search read text inside TIFF images (OCR). The feature's files stay on disk, so turning this OFF re-enables it without downloading anything. | One less component loaded or indexed. | Text in scanned TIFF files isn't searchable. | After a restart |

## Windows features that change or stop working

- **Disable location for all users**: Maps, weather, 'Find my device', automatic time zone and any app that needs your location stop getting it.
- **Disable Geolocation Service**: No app gets your location; Find my device and automatic time zone stop working.
- **Disable Auto Time Zone Updater**: Time zone must be set by hand when travelling.
- **Disable energy-efficient Ethernet**: Slightly higher power use (well under a watt per link). Adapters without the setting are left alone.
- **Disable Wake-on-LAN**: The PC can't be woken remotely (WoL tools, some NAS or backup schedules).
- **Disable NIC vendor power saving**: A little more power use; on laptops on battery that means shorter battery life.
- **Don't let Windows turn off the network adapter**: The adapter stays powered during sleep (slightly higher sleep power on laptops).
- **Shorten TCP TIME_WAIT to 30 seconds**: In theory a very late packet of an old connection could be mixed up with a new one; harmless in normal use.
- **Disable LLMNR**: Old devices on the LAN reachable only by LLMNR name stop resolving by name; use their IP or DNS name.
- **Disable smart multi-homed name resolution**: If the first adapter's DNS is slow or broken, Windows waits instead of racing another adapter.
- **Don't cache failed DNS lookups**: Names that really don't exist are asked again every time (negligible traffic).
- **Disable WAP Push Message Routing**: Work/school device enrollment (Intune/MDM) stops working.
- **Disable Program Compatibility Assistant**: No automatic compatibility help for legacy software.
- **Disable Game DVR & Broadcast User Service**: Game Bar recording (Win+Alt+R) and clips stop working.
- **Disable Phone Service**: Calls through Phone Link and some VoIP apps may stop working.
- **Disable Payments and NFC/SE Manager**: NFC payments stop (most desktops have no NFC).
- **Disable Messaging Service**: SMS/MMS in Phone Link may stop.
- **Disable Contact Data Indexing**: Contact search in Mail/People may be slower or incomplete.
- **Disable Mobile Hotspot Service**: Mobile Hotspot can't be turned on.
- **Disable Link-Layer Topology Discovery Mapper**: The network map feature has nothing to show.
- **Disable SSDP Discovery**: UPnP devices stop appearing in File Explorer > Network; casting to some DLNA TVs stops.
- **Disable UPnP Device Host**: This PC can't act as a UPnP/DLNA device.
- **Disable Function Discovery Resource Publication**: Other PCs stop seeing this PC under Network.
- **Disable TCP/IP NetBIOS Helper**: NetBIOS name resolution and old-style network browsing stop.
- **Disable Sensor Monitoring Service**: Adaptive brightness from the light sensor stops.
- **Disable Sensor Data Service**: Apps can't read sensor data.
- **Disable Parental Controls**: Family Safety limits and activity reports aren't enforced on this PC.
- **Disable Distributed Link Tracking Client**: Shortcuts to moved files may break instead of being repaired.
- **Disable Still Image Acquisition Events**: Pressing Scan on a device no longer starts an app.
- **Disable Application Layer Gateway**: Legacy protocol helpers for ICS (e.g. old FTP) stop.
- **Disable Certificate Propagation**: Smart-card certificates aren't added automatically.
- **Disable Diagnostic Execution Service**: Troubleshooters that run scripts can't run.
- **Disable Recommended Troubleshooting**: No automatic recommended troubleshooting.
- **Disable Intel Management Engine Local Management**: Intel AMT remote management (business PCs) stops; consumer PCs don't use it.
- **Disable FireWire (IEEE 1394) Controller**: FireWire devices (old cameras, audio interfaces) stop working.
- **Disable the family safety monitor task**: Family Safety activity monitoring stops.
- **Disable the family safety refresh task**: Family Safety settings don't refresh.
- **Block device companion software auto-install**: Some devices may install with reduced features until you install the vendor software yourself.
- **Skip the lock screen**: No lock-screen picture, clock, notifications or widgets; you still have to sign in.
- **Disable Game Bar captures (your account)**: Game Bar recording, screenshots and clips stop working for your account.
- **Allow global timer resolution requests**: Higher idle power use and shorter battery life while any program requests a fine timer. No effect unless something requests one.
- **Disable SysMain (Superfetch)**: Frequently used apps can start slower, especially from a hard disk; boot prefetching stops. Memory compression can no longer be changed while it is off.
- **Disable page combining**: Slightly higher memory use.
- **Keep the kernel and drivers in RAM**: A few hundred MB less RAM for apps; little measurable benefit on PCs with plenty of RAM.
- **Stop creating 8.3 short file names**: Very old 16-bit/DOS-era installers and tools that need short names may fail on new files.
- **Give NTFS more cache memory**: More kernel memory use; little effect on typical use.
- **Detect 'Not responding' apps sooner**: Apps that briefly freeze (loading large files) are labelled 'Not responding' more often.
- **No drop shadows on desktop icon labels**: Labels can be harder to read on busy wallpapers.
- **Don't let Store apps run in the background**: Store apps (Mail, Calendar, Phone Link, some games) can't send live notifications or sync while closed.
- **Remove work folders client**: You can't sync with a Work Folders server.
- **Remove internet printing client**: IPP/URL printers can't be added; USB and normal network printers still work.
- **Remove XPS services**: The 'Microsoft XPS Document Writer' printer disappears.
- **Remove TIFF iFilter**: Text in scanned TIFF files isn't searchable.

## Compatibility

- Made for Windows 10 and 11. A change that doesn't apply to your Windows version is skipped and shown as not supported.
- Cheetix must run as administrator to apply system changes.
- Power settings in these presets only change the plugged-in values; battery behavior on laptops is left alone.
- Some optional Windows features are removed. Turning the change off adds them back, which can take a few minutes.

## Undoing it

Before each change, Cheetix saves the exact original setting. Every change is listed in **History**, and turning it off there (or on its page in Advanced Tweaks) writes the original back. Changes marked "after a restart" need one more restart to go back as well.
