# Moderate Tweaks

*A Cheetix preset · for: More noticeable optimization*

Everything in Minor, plus performance and power settings for when the PC is plugged in, Edge clean-up, and rarely used services such as Fax and error reporting switched off.

**95 changes in total.** This preset includes all 36 changes of [Minor Tweaks](minor.md) and adds the 59 below.

Nothing is applied until you review the list in Cheetix and press **Apply**. Changes that are already on are left alone, and changes your version of Windows doesn't support are skipped.

## What it changes

### Privacy & Telemetry

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable Windows error reporting** | Stops Windows from collecting crash and hang reports for apps and sending them to Microsoft. | No report uploads or WerFault.exe report processing after crashes. | Microsoft's 'solutions to problems' stop appearing and developers get no crash reports from you. Blue-screen dump files are still written locally. | Right away |
| **Disable inking & typing personalisation** | Stops Windows from collecting your handwriting and typing patterns (and harvesting your contacts' names) to build a personal dictionary. | Less typing data collected. | Handwriting recognition and text suggestions learn less from you, so they may be less accurate. | After signing out |
| **Disable offline maps auto-update** | Stops Windows from automatically downloading updates for offline maps (Settings > Apps > Offline maps). | No automatic map downloads over your connection. | Offline maps you downloaded go out of date until you update them by hand. | Right away |
| **Disable Wi-Fi session trace** | Stops the always-on 'WiFiSession' trace that records Wi-Fi connection events from starting at boot (sets its Autologger 'Start' value to 0). The logger's registry key is left in place, so turning this off puts it back exactly. | One less trace session and its small, constant disk writes. | The Wi-Fi report (netsh wlan show wlanreport) and network troubleshooter have less history. | After a restart |
| **Disable network lightweight trace** | Stops the always-on 'LwtNetLog' network diagnostics trace from starting at boot (sets its Autologger 'Start' value to 0). The logger's registry key is left in place, so turning this off puts it back exactly. | One less trace session and its small, constant disk writes. | The network troubleshooter has less data to work with. | After a restart |

### DNS & Name Resolution

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Allow DNS-over-HTTPS** | Sets the 'Configure DNS over HTTPS name resolution' policy to Allow DoH. When your DNS server is one Windows knows supports DoH (Cloudflare 1.1.1.1, Google 8.8.8.8, Quad9 9.9.9.9 and others), lookups go encrypted over HTTPS; otherwise normal DNS is used. | Your ISP and others on the network can't read or alter lookups sent to supported servers. | No effect while you use your ISP's DNS servers. Networks that filter DNS (school, work, parental controls) may stop seeing your lookups. | After a restart |

### Security

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Remove SMB 1.0 support** | Disables the 'SMB 1.0/CIFS File Sharing Support' optional feature (client and server parts). | Removes that attack surface. | Very old NAS boxes, routers with USB sharing and Windows XP machines that only speak SMB1 can't be reached. | After a restart |
| **Remove PowerShell 2.0 engine** | Disables the legacy Windows PowerShell 2.0 engine that still ships beside 5.1 on many installs. | Closes that downgrade path. | Ancient scripts that need -Version 2 stop working. Windows 11 24H2 removed the engine, so it shows as not supported there. | After a restart |

### Windows Services

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable Diagnostics Hub Collector** | Collects real-time ETW traces for Visual Studio's diagnostic tools. This tweak sets the 'diagnosticshub.standardcollector.service' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Visual Studio's Diagnostic Tools and profiler can't collect data. | After a restart |
| **Disable Downloaded Maps Manager** | Manages offline maps for the Maps app. This tweak sets the 'MapsBroker' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Offline maps can't be downloaded or updated. | After a restart |
| **Disable Retail Demo Service** | Only used when a PC is put into store-display demo mode. This tweak sets the 'RetailDemo' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | None outside retail demo mode. | After a restart |
| **Disable Windows Error Reporting Service** | Processes crash and hang reports. This tweak sets the 'WerSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Crash reports aren't processed or sent; 'check for solutions' does nothing. | After a restart |
| **Disable Problem Reports Control Panel Support** | Backs the Control Panel 'Problem Reports' page. This tweak sets the 'wercplsupport' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | That page can't list or clear reports. | After a restart |
| **Disable Fax Service** | Sends and receives faxes through a fax modem. This tweak sets the 'Fax' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Windows Fax and Scan can't fax. | After a restart |
| **Disable Windows Media Player Network Sharing** | Shares the Windows Media Player library with other devices. This tweak sets the 'WMPNetworkSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | WMP Legacy can't stream its library to other devices. | After a restart |
| **Disable Wallet Service** | Hosts objects used by Microsoft Wallet clients. This tweak sets the 'WalletService' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Wallet clients can't store payment data (rarely used on PCs). | After a restart |
| **Disable AllJoyn Router** | Routes AllJoyn messages for IoT devices. This tweak sets the 'AJRouter' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | AllJoyn devices (rare, mostly old smart-home gear) can't talk to this PC. | After a restart |
| **Disable BranchCache** | Caches content from servers for corporate networks. This tweak sets the 'PeerDistSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | BranchCache stops (not present on Home). | After a restart |
| **Disable Peer Networking Grouping** | Multi-party communication for Windows Peer Networking. This tweak sets the 'p2psvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Apps using Windows Peer Networking (rare) stop working. | After a restart |
| **Disable Peer Name Resolution Protocol** | Serverless name resolution for peer networking. This tweak sets the 'PNRPsvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Apps using PNRP (rare) stop working. | After a restart |
| **Disable Inventory and Compatibility Appraisal** | Collects an inventory of installed apps and drivers for upgrade-compatibility checks and telemetry. This tweak sets the 'InventorySvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Feature-upgrade compatibility checks may be less accurate. | After a restart |
| **Disable Windows Health and Optimized Experiences** | Provides 'health and optimized experience' recommendations. This tweak sets the 'whesvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Those recommendations stop. | After a restart |
| **Disable Graphics Perf Diagnostic** | Collects GPU performance diagnostics for tools. This tweak sets the 'GraphicsPerfSvc' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | GPU diagnostics for some tools stop; games are unaffected. | After a restart |
| **Disable Offline Files** | Keeps offline copies of network files (Sync Center). This tweak sets the 'CscService' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | Offline Files/Sync Center stop working. | After a restart |
| **Disable PC Speaker Beep Driver** | Kernel driver for the motherboard speaker beep. This tweak sets the 'beep' service to Disabled. | Slightly less memory and background activity; in most cases a small, not measurable, gain. | No motherboard-speaker beeps from Windows. | After a restart |

### Scheduled Tasks

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable the compatibility appraiser task** | Scans installed programs daily and sends compatibility telemetry; can use noticeable CPU and disk while it runs. This tweak disables the scheduled task '\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | Feature upgrades are offered without checking your apps first. | Right away |
| **Disable the startup app impact scan task** | Measures how much each startup app slows sign-in. This tweak disables the scheduled task '\Microsoft\Windows\Application Experience\StartupAppTask'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | Task Manager's 'Startup impact' column may show 'Not measured'. | Right away |
| **Disable the disk diagnostic data collector task** | Collects disk diagnostic information and reports it to Microsoft. This tweak disables the scheduled task '\Microsoft\Windows\DiskDiagnostic\Microsoft-Windows-DiskDiagnosticDataCollector'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | Windows' own disk-failure diagnostics have less data; SMART tools are unaffected. | Right away |
| **Disable the queued error reporting task** | Sends queued crash reports. This tweak disables the scheduled task '\Microsoft\Windows\Windows Error Reporting\QueueReporting'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | Queued crash reports are not sent. | Right away |
| **Disable the maps update task** | Checks for offline map updates. This tweak disables the scheduled task '\Microsoft\Windows\Maps\MapsUpdateTask'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | Offline maps aren't updated automatically. | Right away |
| **Disable the maps notifications task** | Shows notifications about offline maps. This tweak disables the scheduled task '\Microsoft\Windows\Maps\MapsToastTask'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |
| **Disable the gather network info task** | Collects network configuration for diagnostics. This tweak disables the scheduled task '\Microsoft\Windows\NetTrace\GatherNetworkInfo'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | Network diagnostics have less history. | Right away |

### Windows Debloat

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Don't download device metadata** | Policy that stops Windows downloading manufacturer device info, icons and companion-app links for connected devices. | No device metadata downloads. | Devices and Printers shows generic icons and names. | Right away |
| **Disable AI features in Notepad** | Policy that turns off Copilot-powered features (Rewrite, Summarize) in the new Notepad. | No AI buttons in Notepad. | No AI rewrite/summarize in Notepad. | When the app restarts |

### Microsoft Edge

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable shopping assistant** | Removes Edge's price-comparison and coupon pop-ups. (policy EdgeShoppingAssistantEnabled = 0) | No shopping pop-ups. | No price comparisons in Edge. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off. | When the app restarts |
| **Edge: hide the sidebar** | Removes the right-hand sidebar (Copilot, tools, games) from Edge. (policy HubsSidebarEnabled = 0) | More room and fewer sidebar processes. | No Copilot/sidebar apps in Edge. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off. | When the app restarts |
| **Disable startup boost** | Stops Edge from launching hidden processes at sign-in so it opens faster. (policy StartupBoostEnabled = 0) | Fewer background Edge processes and less memory use after sign-in. | Edge opens a little slower the first time. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off. | When the app restarts |
| **Disable tips & recommendations** | Stops Edge showing feature recommendations and notifications. (policy ShowRecommendationsEnabled = 0) | Fewer Edge pop-ups. | None. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off. | When the app restarts |
| **Disable browsing-data personalisation** | Stops Edge sending browsing history to Microsoft to personalise ads, news and other services. (policy PersonalizationReportingEnabled = 0) | Less browsing data shared. | Less personalised Microsoft news/ads. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off. | When the app restarts |
| **Disable feedback prompts** | Removes Edge's 'Send feedback' prompts and smile button. (policy UserFeedbackAllowed = 0) | No feedback prompts. | You can't send feedback from Edge. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off. | When the app restarts |
| **Hide new tab news feed** | Hides the Microsoft Start news feed on Edge's New Tab page. (policy NewTabPageContentEnabled = 0) | Lighter, faster New Tab page. | No news on New Tab. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off. | When the app restarts |
| **Edge: skip the first-run setup** | Hides Edge's first-run setup pages (policy HideFirstRunExperience = 1). | Edge opens straight to a page. | You don't get the import/sign-in wizard. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off. | When the app restarts |

### Gaming

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable background recording** | Turns off Game Bar's 'Record what happened' (continuously records the last moments of gameplay). | Frees GPU encoder time and a little CPU/RAM while gaming. | You can't save a clip of something that just happened. | Right away |
| **Enable optimizations for windowed games** | Turns on 'Optimizations for windowed games' (Settings > Display > Graphics), which moves older DirectX 10/11 games running windowed or borderless to the modern flip presentation model. | Lower input latency in borderless windowed mode, and Auto HDR/VRR support for those games. | Rare issues with overlays or capture tools in a few games (you can opt a game out in Graphics settings). This value holds all global graphics options, so it is replaced as a whole and restored as a whole when turned off. | When the app restarts |

### CPU & Processor

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Favour CPU performance (plugged in)** | Sets 'Processor energy performance preference policy' to 0 % for the active power plan on AC power. Modern Intel and AMD CPUs use this hint to decide how quickly to raise clocks. | Snappier response; fewer clock ramp-up delays. | Higher idle power and temperatures, more fan noise. The battery value is left alone. | Right away |
| **Hybrid CPUs: prefer performance cores** | Sets the documented 'Heterogeneous thread scheduling policy' and its short-running-thread counterpart to 'Prefer performant processors' for the active plan on AC power. | Threads go to P-cores first when they are free. | More power use; background work also lands on P-cores. No effect on CPUs without E-cores. | Right away |

### Power Management

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Turn off PCI Express power saving (plugged in)** | Sets 'Link State Power Management' to Off for the active plan on AC power, so PCIe links (GPU, NVMe, network) never drop into low-power states. | Removes those wake-up delays. | A few watts more at idle. | Right away |

### USB & Peripherals

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Turn off USB 3 power saving (plugged in)** | Sets 'USB 3 Link Power Management' to Off for the active plan on AC power. | Steadier USB audio interfaces, capture cards, external SSDs and wireless dongles. | Slightly more power use. | Right away |
| **Disable the Filter Keys shortcut** | Stops holding Right Shift for 8 seconds from turning on Filter Keys (which makes the keyboard ignore quick or repeated presses). | No surprise accessibility pop-up or sluggish keyboard mid-game. | The shortcut can't turn Filter Keys on (Settings still can). | After signing out |
| **Disable the Toggle Keys shortcut** | Stops holding Num Lock for 5 seconds from turning on Toggle Keys (beeps on Caps/Num/Scroll Lock). | No surprise pop-up. | The shortcut can't turn Toggle Keys on (Settings still can). | After signing out |
| **Disable Windows Ink Workspace** | Policy that turns off the pen workspace/pen menu. | No pen menu or its taskbar button. | Pen shortcuts from the pen menu stop; pen input itself still works. | After signing out |

### RAM & Memory

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable app pre-launch** | Stops Windows from starting Store apps (Mail, Photos and others) suspended in the background so they open faster. | Fewer suspended apps in memory. | Those apps open a bit slower. | After a restart |

### Storage & SSD

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Enable long file paths** | Lets programs that support it use paths longer than 260 characters. | Deep folder trees, npm/git projects and archives work. | Programs without long-path support still fail on such paths; nothing breaks. | After a restart |
| **Disable NTFS last-access timestamps** | Stops NTFS updating each file's 'last accessed' time when it is read (0x80000001 = set by the user, disabled). | Fewer writes when reading many files. | Tools that rely on 'last accessed' dates (e.g. Storage Sense's 'not opened for X days') see stale dates. Windows already disables it on large volumes by default. | After a restart |
| **No Thumbs.db on network folders** | Stops File Explorer writing hidden thumbs.db thumbnail caches into network shares. | No stray thumbs.db files and no 'file in use' when deleting network folders. | Thumbnails for network files are generated again each visit. | After Explorer restarts |

### Responsiveness

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Lower the background CPU reserve to 10 %** | The Multimedia Class Scheduler reserves a share of CPU for low-priority work while multimedia threads (audio, games registered with MMCSS) run. The default is 20 %; this sets the documented minimum of 10 %. | Up to 90 % instead of 80 % CPU for those threads. | Background tasks get less CPU while media plays. | After a restart |

### Startup & Background Apps

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **No delay for startup apps** | Windows normally waits about 10 seconds after sign-in before launching startup apps; this starts them immediately. | Chat, sync and mouse software available right away. | More programs compete with Explorer right after sign-in, so the desktop can feel busy for a moment. | After signing out |
| **Don't auto-sign-in after updates** | Policy that stops Windows using your sign-in info to automatically finish setup and lock the PC after an update restart. | Nothing runs as you until you sign in. | After an update restart, apps and settings finish loading only after you sign in. | Right away |

### GPU & Graphics

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable NVIDIA telemetry** | Sets the NVIDIA driver's 'SendTelemetryData' flag to 0, opts out of NVIDIA Control Panel data collection and turns off three NVIDIA feature-telemetry flags (RID44231/64640/66610). | Less data collection; no effect on performance. | None known. | After a restart |

## Windows features that change or stop working

- **Disable Windows error reporting**: Microsoft's 'solutions to problems' stop appearing and developers get no crash reports from you. Blue-screen dump files are still written locally.
- **Disable inking & typing personalisation**: Handwriting recognition and text suggestions learn less from you, so they may be less accurate.
- **Disable offline maps auto-update**: Offline maps you downloaded go out of date until you update them by hand.
- **Disable Wi-Fi session trace**: The Wi-Fi report (netsh wlan show wlanreport) and network troubleshooter have less history.
- **Disable network lightweight trace**: The network troubleshooter has less data to work with.
- **Allow DNS-over-HTTPS**: No effect while you use your ISP's DNS servers. Networks that filter DNS (school, work, parental controls) may stop seeing your lookups.
- **Remove SMB 1.0 support**: Very old NAS boxes, routers with USB sharing and Windows XP machines that only speak SMB1 can't be reached.
- **Remove PowerShell 2.0 engine**: Ancient scripts that need -Version 2 stop working. Windows 11 24H2 removed the engine, so it shows as not supported there.
- **Disable Diagnostics Hub Collector**: Visual Studio's Diagnostic Tools and profiler can't collect data.
- **Disable Downloaded Maps Manager**: Offline maps can't be downloaded or updated.
- **Disable Windows Error Reporting Service**: Crash reports aren't processed or sent; 'check for solutions' does nothing.
- **Disable Problem Reports Control Panel Support**: That page can't list or clear reports.
- **Disable Fax Service**: Windows Fax and Scan can't fax.
- **Disable Windows Media Player Network Sharing**: WMP Legacy can't stream its library to other devices.
- **Disable Wallet Service**: Wallet clients can't store payment data (rarely used on PCs).
- **Disable AllJoyn Router**: AllJoyn devices (rare, mostly old smart-home gear) can't talk to this PC.
- **Disable BranchCache**: BranchCache stops (not present on Home).
- **Disable Peer Networking Grouping**: Apps using Windows Peer Networking (rare) stop working.
- **Disable Peer Name Resolution Protocol**: Apps using PNRP (rare) stop working.
- **Disable Inventory and Compatibility Appraisal**: Feature-upgrade compatibility checks may be less accurate.
- **Disable Windows Health and Optimized Experiences**: Those recommendations stop.
- **Disable Graphics Perf Diagnostic**: GPU diagnostics for some tools stop; games are unaffected.
- **Disable Offline Files**: Offline Files/Sync Center stop working.
- **Disable PC Speaker Beep Driver**: No motherboard-speaker beeps from Windows.
- **Disable the compatibility appraiser task**: Feature upgrades are offered without checking your apps first.
- **Disable the startup app impact scan task**: Task Manager's 'Startup impact' column may show 'Not measured'.
- **Disable the disk diagnostic data collector task**: Windows' own disk-failure diagnostics have less data; SMART tools are unaffected.
- **Disable the queued error reporting task**: Queued crash reports are not sent.
- **Disable the maps update task**: Offline maps aren't updated automatically.
- **Disable the gather network info task**: Network diagnostics have less history.
- **Don't download device metadata**: Devices and Printers shows generic icons and names.
- **Disable AI features in Notepad**: No AI rewrite/summarize in Notepad.
- **Disable shopping assistant**: No price comparisons in Edge. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off.
- **Edge: hide the sidebar**: No Copilot/sidebar apps in Edge. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off.
- **Disable startup boost**: Edge opens a little slower the first time. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off.
- **Disable browsing-data personalisation**: Less personalised Microsoft news/ads. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off.
- **Disable feedback prompts**: You can't send feedback from Edge. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off.
- **Hide new tab news feed**: No news on New Tab. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off.
- **Edge: skip the first-run setup**: You don't get the import/sign-in wizard. Edge shows 'Your browser is managed by your organization' while any Edge policy is set; that banner goes away when the last one is turned off.
- **Disable background recording**: You can't save a clip of something that just happened.
- **Enable optimizations for windowed games**: Rare issues with overlays or capture tools in a few games (you can opt a game out in Graphics settings). This value holds all global graphics options, so it is replaced as a whole and restored as a whole when turned off.
- **Favour CPU performance (plugged in)**: Higher idle power and temperatures, more fan noise. The battery value is left alone.
- **Hybrid CPUs: prefer performance cores**: More power use; background work also lands on P-cores. No effect on CPUs without E-cores.
- **Turn off PCI Express power saving (plugged in)**: A few watts more at idle.
- **Turn off USB 3 power saving (plugged in)**: Slightly more power use.
- **Disable the Filter Keys shortcut**: The shortcut can't turn Filter Keys on (Settings still can).
- **Disable the Toggle Keys shortcut**: The shortcut can't turn Toggle Keys on (Settings still can).
- **Disable Windows Ink Workspace**: Pen shortcuts from the pen menu stop; pen input itself still works.
- **Disable app pre-launch**: Those apps open a bit slower.
- **Enable long file paths**: Programs without long-path support still fail on such paths; nothing breaks.
- **Disable NTFS last-access timestamps**: Tools that rely on 'last accessed' dates (e.g. Storage Sense's 'not opened for X days') see stale dates. Windows already disables it on large volumes by default.
- **No Thumbs.db on network folders**: Thumbnails for network files are generated again each visit.
- **Lower the background CPU reserve to 10 %**: Background tasks get less CPU while media plays.
- **No delay for startup apps**: More programs compete with Explorer right after sign-in, so the desktop can feel busy for a moment.
- **Don't auto-sign-in after updates**: After an update restart, apps and settings finish loading only after you sign in.

## Compatibility

- Made for Windows 10 and 11. A change that doesn't apply to your Windows version is skipped and shown as not supported.
- Cheetix must run as administrator to apply system changes.
- Power settings in these presets only change the plugged-in values; battery behavior on laptops is left alone.

## Undoing it

Before each change, Cheetix saves the exact original setting. Every change is listed in **History**, and turning it off there (or on its page in Advanced Tweaks) writes the original back. Changes marked "after a restart" need one more restart to go back as well.
