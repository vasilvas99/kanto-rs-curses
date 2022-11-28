# Kanto CM curses

An attempt at a TUI for kanto-cm

![Screenshot](/misc/kantocmcursesss.png)

Requires root to bind to kanto-cm unix socket. To capture stderr in a file since ncurses otherwise cleans-up the terminal after a crash:
```bash
RUST_BACKTRACE=1 sudo kantocurses 2> stderr.log
```