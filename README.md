# rrplay

Vim inspired music player written in rust (work in progress).\
Quit it like you would quit vim, navigate it like you would navigate Vim.\
Aims for fast navigation which i missed from most players.

# Sources

(Still wip in v2, please manually make your config folder and the file for now)

You are prompted to choose your source folder when first opening the program.\
example:

```bash
/home/your_name/Music
```

Multiple sources can be added simply by listing each one on a seperate line.\
Sources will be colored either green or red depending whether they are valid or
not.

# Navigation:

### Keybinds

**Modes:**\
: Command mode\
/ Search mode\
(Enter) Select mode (While being in search mode)\
(Esc) Sitback mode (Just shows your queue and other info)

**General:**\
p - Pause\
s - Skip song\
V - Volume up\
v - Volume down\
c - Clear queue\
h - move backwards 5s\
l - move forward 5s

**Select mode:**\
Enter - Add single to queue\
a - Add album to queue

### Searching

Fuzzy search allows you to quikcly find what you are looking for whether its an
album, a track, an artist or a mix of everything since it uses the full paths.

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
