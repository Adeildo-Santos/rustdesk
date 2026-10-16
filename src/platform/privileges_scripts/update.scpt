on run {daemon_file, agent_file, user, cur_pid, source_dir}

  -- ATENCAO: caminhos entre aspas (quoted form): o nome do app tem espaco
  -- ("Ecletica Connect") e sem aspas o shell quebra o nome no meio.
  set agent_plist_path to "/Library/LaunchAgents/com.carriez.RustDesk_server.plist"
  set daemon_plist_path to "/Library/LaunchDaemons/com.carriez.RustDesk_service.plist"
  set agent_plist to quoted form of agent_plist_path
  set daemon_plist to quoted form of daemon_plist_path
  set app_bundle to quoted form of "/Applications/RustDesk.app"

  set check_source to "test -d " & quoted form of source_dir & " || exit 1;"
  set resolve_uid to "uid=$(id -u " & quoted form of user & " 2>/dev/null || true);"
  set unload_agent to "if [ -n \"$uid\" ]; then launchctl bootout gui/$uid " & agent_plist & " 2>/dev/null || launchctl bootout user/$uid " & agent_plist & " 2>/dev/null || launchctl unload -w " & agent_plist & " || true; else launchctl unload -w " & agent_plist & " || true; fi;"
  set unload_service to "launchctl unload -w " & daemon_plist & " || true;"
  set kill_others to "pids=$(pgrep -x 'RustDesk' | grep -vx " & cur_pid & " || true); if [ -n \"$pids\" ]; then echo \"$pids\" | xargs kill -9 || true; fi;"

  set copy_files to "(rm -rf " & app_bundle & " && ditto " & quoted form of source_dir & " " & app_bundle & " && chown -R " & quoted form of user & ":staff " & app_bundle & " && (xattr -r -d com.apple.quarantine " & app_bundle & " || true)) || exit 1;"

  set write_daemon_plist to "echo " & quoted form of daemon_file & " > " & daemon_plist & " && chown root:wheel " & daemon_plist & ";"
  set write_agent_plist to "echo " & quoted form of agent_file & " > " & agent_plist & " && chown root:wheel " & agent_plist & ";"
  set load_service to "launchctl load -w " & daemon_plist & ";"
  set agent_label_cmd to "agent_label=$(basename " & agent_plist & " .plist);"
  set bootstrap_agent to "if [ -n \"$uid\" ]; then launchctl bootstrap gui/$uid " & agent_plist & " 2>/dev/null || launchctl bootstrap user/$uid " & agent_plist & " 2>/dev/null || launchctl load -w " & agent_plist & " || true; else launchctl load -w " & agent_plist & " || true; fi;"
  set kickstart_agent to "if [ -n \"$uid\" ]; then launchctl kickstart -k gui/$uid/$agent_label 2>/dev/null || launchctl kickstart -k user/$uid/$agent_label 2>/dev/null || true; fi;"
  set load_agent to agent_label_cmd & bootstrap_agent & kickstart_agent

  set sh to "set -e;" & check_source & resolve_uid & unload_agent & unload_service & kill_others & copy_files & write_daemon_plist & write_agent_plist & load_service & load_agent

  do shell script sh with prompt "RustDesk wants to update itself" with administrator privileges
end run
