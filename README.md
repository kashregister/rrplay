# rrplay

Vim inspired music player written in rust (work in progress).\
Quit it like you would quit vim, navigate it like you would navigate Vim.\
Aims for fast navigation which i missed from most players.

# Sources

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
**Select mode:**\
Enter - Play single\
a - Play album

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
to be added

```
```
