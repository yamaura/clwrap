Command line wrapper for autologin then execute command

# How to use

Double dash(--) separated two commands, 1st is spawn commnd to open terminal and then execute command in opened terminal.

For example, use telnet to open terminal then type `whoami` command in it.
```
$ clwrap -u [username_to_login] -p [password_to_login] -- telnet [target_host] -- whoami
username_to_login
```

# Compile

If you want to use clwrap command. You have to enable `--features="cmdline"` option.
