# rrplay

vim inspired music player written in rust (work in progress) \
aims for fast navigation which i missed from most players
# sources
you are prompted to choose your source when first opening the program. \
example: 
```
/home/kr24/Music
```
you can add multiple sources simply by adding each one on a seperate line\
sources will be colored either green or red depending whether they are valid or not
# navigation:
### modes
: - command mode \
/ - search mode \
enter - select mode (after being in search mode)

### actions
p - pause \
a - play album (in select mode)
enter - play single song (in select mode)

### searching
fuzzy search allows you to quikcly find what you are looking for whether its an album, a track, an artist or a mix of everything since it uses the full paths

quit it like you would quit vim, 
navigate it like you would navigate vim

# installing

there is a small script budnled in to move it to your bin directory after compiling 
```
chmod +x build_and_mv.sh
sh build_and_mv.sh
```
or if you prefer doing it manually
```
cargo build --release
```

