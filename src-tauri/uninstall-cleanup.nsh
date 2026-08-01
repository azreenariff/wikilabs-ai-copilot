!macro NSIS_HOOK_POSTUNINSTALL
  ; Remove app data directory on uninstall
  ; This ensures logs, settings, and cached data are cleaned up
  RMDir /r "$APPDATA\com.wikilabs.copilot"
  RMDir /r "$LOCALAPPDATA\com.wikilabs.copilot"
  RMDir /r "$APPDATA\wikilabs-ai-copilot"
  ; Remove start menu shortcut folder if empty
  RMDir "$SMPROGRAMS\Wiki Labs AI Copilot"
!macroend