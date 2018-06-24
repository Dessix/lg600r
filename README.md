`lg600r` is a simple Linux utility for listening to a Logitech G600 mouse and firing off commands on keypress.

To use:
- Using Windows or Mac Logitech Gaming Software, bind unique keys to every non-click button, including GShift
- Create a dotfile following the provided example
- Place it at `~/.config/lg600r/config.toml` or `~/.lg600r/config.toml`
- Run the executable
- Press buttons on your mouse while looking at the output to identify which scancodes associate with which button.
- Edit the dotfile, restart the executable, and enjoy :)


This project was inspired by [mafik/logitech-g600-linux](https://github.com/mafik/logitech-g600-linux).
