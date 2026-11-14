# Minor Tweaks

*A Cheetix preset · for: Beginners and everyday PCs*

Turns off telemetry, advertising, tips and suggested-app installs. Lightweight changes: nothing you normally use stops working.

**36 changes in total.** All of them are low risk.

Nothing is applied until you review the list in Cheetix and press **Apply**. Changes that are already on are left alone, and changes your version of Windows doesn't support are skipped.

## What it changes

### Privacy & Telemetry

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable activity history (policy)** | Group-policy switch that stops Windows from recording which apps, files and websites you open (Activity History) and from uploading that history to your Microsoft account. | Less personal data stored and synced; removes a small amount of background logging. | 'Pick up where you left off' suggestions and cross-device activity resume stop working. | After signing out |
| **Block feedback notifications (policy)** | Policy that stops Windows and the Feedback Hub from showing 'How likely are you to recommend Windows…' survey pop-ups for every user. | No survey interruptions. | None; you can still open Feedback Hub yourself. | Right away |
| **Never ask for feedback** | Sets your account's 'Feedback frequency' (Settings > Privacy > Diagnostics & feedback) to Never by setting the survey count to 0 and removing the survey period. | No feedback requests for your account. | None. | Right away |
| **Disable advertising ID (all users)** | Policy that turns off the advertising ID apps use to show personalised ads, for every account on this PC. | Ads in apps become non-personalised. | None beyond less relevant ads. | After signing out |
| **Disable tailored experiences** | Policy that stops Microsoft from using your diagnostic data to personalise tips, ads and recommendations in Windows. | Fewer personalised promotions. | Tips and suggestions become generic. | After signing out |
| **Disable typing insights** | Turns off 'Typing insights' (Settings > Time & language > Typing), which records statistics about your typing and autocorrect use. | Less input data collected. | The Typing insights page shows no data. | Right away |
| **Limit diagnostic log collection** | Policy that stops Windows from collecting additional diagnostic logs when your diagnostic level is Optional. | Less data uploaded. | Microsoft gets less information to diagnose problems on your PC. | Right away |
| **Limit crash-dump collection** | Policy that limits crash dumps sent with optional diagnostic data to small 'mini' dumps. | Less sensitive data uploaded. | Microsoft gets less detail about crashes. | Right away |
| **Disable DiagTrack boot logger** | Stops the 'Diagtrack-Listener' trace session that starts at boot to buffer telemetry events for the Connected User Experiences (DiagTrack) service from starting at boot (sets its Autologger 'Start' value to 0). The logger's registry key is left in place, so turning this off puts it back exactly. | One less trace session and its small, constant disk writes. | None for normal use; telemetry events from early boot are not collected. | After a restart |
| **Disable SQM logger** | Stops the 'SQMLogger' session that records Software Quality Metrics (legacy Customer Experience Improvement Program data) from starting at boot (sets its Autologger 'Start' value to 0). The logger's registry key is left in place, so turning this off puts it back exactly. | One less trace session and its small, constant disk writes. | None for normal use. | After a restart |
| **Disable setup telemetry logger** | Stops the 'SetupPlatformTel' session that records telemetry about Windows setup and feature upgrades from starting at boot (sets its Autologger 'Start' value to 0). The logger's registry key is left in place, so turning this off puts it back exactly. | One less trace session and its small, constant disk writes. | Upgrade problems are harder for Microsoft to diagnose. | After a restart |
| **Disable OOBE experience logger** | Stops the 'CloudExperienceHostOobe' session that logs the out-of-box and first-sign-in screens from starting at boot (sets its Autologger 'Start' value to 0). The logger's registry key is left in place, so turning this off puts it back exactly. | One less trace session and its small, constant disk writes. | None for normal use. | After a restart |

### Network

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable peer-to-peer update sharing** | Policy that makes Delivery Optimization download Windows and Store updates from Microsoft only (HTTP mode), never from or to other PCs. Updates themselves keep working. | Your upload bandwidth is never used to serve updates. | On a network with several Windows PCs each one downloads updates separately. | Right away |

### Security

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Block Remote Assistance invitations** | Unticks 'Allow Remote Assistance connections to this computer' (System Properties > Remote). | Nobody can take control through a Remote Assistance invitation. | You can't receive Remote Assistance (msra.exe) help. Quick Assist and Remote Desktop are separate. | Right away |
| **Disable autorun on all drives** | Policy that stops Windows from automatically running anything (autorun.inf) from USB sticks, discs or network drives. | Inserted media can't start programs by itself. | Installer discs and some USB gadgets no longer start automatically; open them from File Explorer. | After signing out |

### Scheduled Tasks

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Disable the program data updater task** | Collects program telemetry for the Customer Experience Improvement Program (removed in newer builds). This tweak disables the scheduled task '\Microsoft\Windows\Application Experience\ProgramDataUpdater'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |
| **Disable the compatibility data backup task** | Backs up compatibility-appraiser data for telemetry. This tweak disables the scheduled task '\Microsoft\Windows\Application Experience\MareBackup'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |
| **Disable the Autochk SQM proxy task** | Collects and uploads quality metrics from Autochk. This tweak disables the scheduled task '\Microsoft\Windows\Autochk\Proxy'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |
| **Disable the CEIP consolidator task** | Collects and sends Customer Experience Improvement Program data. This tweak disables the scheduled task '\Microsoft\Windows\Customer Experience Improvement Program\Consolidator'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |
| **Disable the USB CEIP task** | Collects USB usage statistics for Microsoft. This tweak disables the scheduled task '\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |
| **Disable the feedback data client task** | Downloads and runs feedback/survey scenarios. This tweak disables the scheduled task '\Microsoft\Windows\Feedback\Siuf\DmClient'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |
| **Disable the feedback scenario download task** | Runs feedback scenarios when they are downloaded. This tweak disables the scheduled task '\Microsoft\Windows\Feedback\Siuf\DmClientOnScenarioDownload'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |
| **Disable the device census task** | Collects hardware and OS details for telemetry. This tweak disables the scheduled task '\Microsoft\Windows\Device Information\Device'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |
| **Disable the device census (user) task** | Collects per-user device information for telemetry. This tweak disables the scheduled task '\Microsoft\Windows\Device Information\Device User'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |
| **Disable the SQM tasks** | Uploads Software Quality Metrics. This tweak disables the scheduled task '\Microsoft\Windows\PI\Sqm-Tasks'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |
| **Disable the power efficiency diagnostics task** | Analyses power use and reports it. This tweak disables the scheduled task '\Microsoft\Windows\Power Efficiency Diagnostics\AnalyzeSystem'; turning it off re-enables the task if it was enabled before. | Fewer scheduled background runs (CPU/disk/network) - usually a small gain. | None. | Right away |

### Windows Debloat

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Stop silent installs of suggested apps** | Stops the Content Delivery Manager from quietly installing promoted apps and games (Candy Crush, TikTok and similar) for your account. | No surprise app installs. | None - apps you install yourself are unaffected. | After signing out |
| **Hide app suggestions in Start** | Turns off 'Show suggestions occasionally in Start' (promoted apps). | Cleaner Start menu. | None. | After Explorer restarts |
| **Disable 'Tips and suggestions' notifications** | Turns off 'Get tips and suggestions when using Windows'. | Fewer notifications. | None. | Right away |
| **Hide fun facts & tips on the lock screen** | Removes the 'fun facts, tips, tricks and more' text and links shown on the lock screen. | No promotional text on the lock screen. | None. | Right away |
| **Disable 'Finish setting up your device' screen** | Stops the full-screen 'Let's finish setting up your device' / 'Get even more out of Windows' prompt after updates. | Straight to the desktop after updates. | None. | Right away |
| **Disable Microsoft consumer experiences** | Policy 'Turn off Microsoft consumer experiences': stops Windows from adding third-party app suggestions and promoted apps for users. | Fewer promoted apps for new accounts. | None. | After signing out |
| **Hide account notifications in Start** | Stops Start from showing Microsoft-account prompts such as 'Back up your files' or 'Sign in' badges. | Cleaner Start menu. | None. | After Explorer restarts |
| **Hide Meet Now (Windows 10)** | Removes the Skype 'Meet Now' icon from the Windows 10 taskbar. | Cleaner taskbar. | None. | After Explorer restarts |
| **Disable news and interests (Windows 10)** | Policy that turns off the Windows 10 taskbar weather/news feed. | No news process in the background. | No weather on the taskbar. | After Explorer restarts |

### Microsoft Edge

| Change | What it does | Benefit | Drawback | Takes effect |
|---|---|---|---|---|
| **Edge: no desktop shortcut after updates** | Policy that stops Edge updates from re-creating the Edge shortcut on the desktop. | Clean desktop. | None. | Right away |

## Windows features that change or stop working

- **Disable activity history (policy)**: 'Pick up where you left off' suggestions and cross-device activity resume stop working.
- **Disable tailored experiences**: Tips and suggestions become generic.
- **Disable typing insights**: The Typing insights page shows no data.
- **Limit diagnostic log collection**: Microsoft gets less information to diagnose problems on your PC.
- **Limit crash-dump collection**: Microsoft gets less detail about crashes.
- **Disable setup telemetry logger**: Upgrade problems are harder for Microsoft to diagnose.
- **Disable peer-to-peer update sharing**: On a network with several Windows PCs each one downloads updates separately.
- **Block Remote Assistance invitations**: You can't receive Remote Assistance (msra.exe) help. Quick Assist and Remote Desktop are separate.
- **Disable autorun on all drives**: Installer discs and some USB gadgets no longer start automatically; open them from File Explorer.
- **Disable news and interests (Windows 10)**: No weather on the taskbar.

## Compatibility

- Made for Windows 10 and 11. A change that doesn't apply to your Windows version is skipped and shown as not supported.
- Cheetix must run as administrator to apply system changes.
- Power settings in these presets only change the plugged-in values; battery behavior on laptops is left alone.

## Undoing it

Before each change, Cheetix saves the exact original setting. Every change is listed in **History**, and turning it off there (or on its page in Advanced Tweaks) writes the original back. Changes marked "after a restart" need one more restart to go back as well.
