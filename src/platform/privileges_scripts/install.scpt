on run {daemon_file, agent_file, user}

  -- Pasta de dados do app no macOS. Nome CONGELADO ("Ecletica Acesso Remoto"):
  -- trocar aqui daria ID novo. Nao usa o espaco de nomes "RustDesk" de proposito,
  -- para o correct_app_name() do Rust nao mexer neste caminho.
  set prefs_dir to "/Users/" & user & "/Library/Preferences/com.carriez.Ecletica Acesso Remoto/"
  set prefs_toml to quoted form of (prefs_dir & "RustDesk.toml")
  set prefs2_toml to quoted form of (prefs_dir & "RustDesk2.toml")

  -- ATENCAO: todo caminho abaixo PRECISA ir entre aspas (quoted form): o nome do
  -- app tem espaco ("Ecletica Connect") e o shell quebraria o nome no meio.
  set daemon_plist to quoted form of "/Library/LaunchDaemons/com.carriez.RustDesk_service.plist"
  set agent_plist to quoted form of "/Library/LaunchAgents/com.carriez.RustDesk_server.plist"
  set daemon_label to quoted form of "system/com.carriez.RustDesk_service"

  set sh1 to "echo " & quoted form of daemon_file & " > " & daemon_plist & " && chown root:wheel " & daemon_plist & ";"

  set sh2 to "echo " & quoted form of agent_file & " > " & agent_plist & " && chown root:wheel " & agent_plist & ";"

  set sh3 to "cp -rf " & prefs_toml & " " & quoted form of "/var/root/Library/Preferences/com.carriez.Ecletica Acesso Remoto/;"

  set sh4 to "cp -rf " & prefs2_toml & " " & quoted form of "/var/root/Library/Preferences/com.carriez.Ecletica Acesso Remoto/;"

  set sh5 to "launchctl bootout " & daemon_label & " 2>/dev/null || launchctl unload -w " & daemon_plist & " 2>/dev/null || true; launchctl bootstrap system " & daemon_plist & " 2>/dev/null || launchctl load -w " & daemon_plist & ";"

  set sh to sh1 & sh2 & sh3 & sh4 & sh5

  do shell script sh with prompt "RustDesk wants to install daemon and agent" with administrator privileges
end run
