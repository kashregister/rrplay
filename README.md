# rrplay

Vim inspired music player written in rust (work in progress).\
Quit it like you would quit vim, navigate it like you would navigate Vim.\
Aims for fast navigation which i missed from most players.

# Sources

Each time you start up rrplay, a popup window will list your sources and the
location of the configuration file.

Multiple sources can be added simply by listing each one on a seperate line.\
Sources will be colored either green or red depending whether they are valid or
not.

```bash
/home/your_name/Music
/home/your_name/Music2
/home/your_name/My_folder
```

# Navigation:

### Keybinds

**Modes:**\
/ Search mode\
(Enter) Select mode (While being in search mode)\
(Esc) Sitback mode (Queue and related)\
: Help desk (Popup that lists your sources)

**General:**\
p - Pause\
s - Skip song\
V - Volume up\
v - Volume down\
c - Clear queue\
h - move backwards 5s\
l - move forward 5s

**Select mode:**\
Enter - Add single to the queue\
a - Add album to the queue\
1-6 - Search through different metadata

# Installing

There is a small script budnled in to move it to your bin directory after
compiling:

```bash
chmod +x build_and_mv.sh
sh build_and_mv.sh
```

Or if you prefer doing it manually:

```bash
cargo build --release
```

Nix\
Using flakes:

```nix
inputs = {
    rrplay.url = "github:kashregister/rrplay";
  };
```

```nix
outputs = inputs @ {
  self,
  rrplay,
  ...
}:
```

```nix
home.packages = with pkgs; [
  inputs.rrplay.packages.${pkgs.system}.default
];
```
