# DFVStritus
The DFVStritus framework is a solver for the Directed Feedback Vertex Set Problem. The problem states that given a graph G = (V, E), we
compute a minimum-sized subset X of V such that the induced graph of G[V \ X] is acyclic. This solver was submitted to the PACE Challange 2022 and is written for my master thesis. 

## Installation

## Dependencies
The project relies on interfacing with the [Coin-or CBC library](https://github.com/coin-or/COIN-OR-OptimizationSuite)
* [Coin-or cbc library](https://github.com/coin-or/COIN-OR-OptimizationSuite), which can be installed with apt using 
    
    ```$ apt-get install coinor-libcbc-dev```.

Other main dependencies are fetched and compiled using cargo. These dependencies are `rustc-hash`, `rand`, `shh` (to force cbc to be quiet), and `coin_cbc` for the rust bindings to cbc.

For development purposes, cargo also installs `cpu_time`, `rayon` and `assert_cmd`. These dependencies are not strictly necessary for the solver.

## Building
The solver can be built using `cargo build --release`. 

# Optil
Unfortunately, the optil.io server is running on Ubuntu 16.04 (last checked in April 2022), which comes with the caveat that building the project on your current favorite linux distro compiles and links the binaries against newer versions of `glibc` and `coinor-libcbc-dev`. To remedy this, we use a docker container that is running Ubuntu 16.04, we build the binary in that environment, and extract it. This ensures the binary is linked to the correct versions of `glibc` and `coinor-libcbc-dev` without the need of creating a VM.

### Requirements
* [Podman](https://podman.io/getting-started/installation)

### Building
The binary can be built using 
```
$ ./build_optil.sh
```
It will create the directory `optil_target/` which contains both the binary and a tarball which can be submitted directly to the platform. 

## Miscellaneous: The name of the framework 

I am a big fan of [Sir Terry Pratchett's](https://en.wikipedia.org/wiki/Terry_Pratchett) famous [Discworld](https://en.wikipedia.org/wiki/Discworld) Series, they're splendid and I highly recommend reading the series if you have time for 41 books. Throughout the series, Sir Terry made a number of references to computing devices, such as [Hex](https://en.wikipedia.org/wiki/Hex_(Discworld)), or incredibly clever characters, such as [You Bastard](https://wiki.lspace.org/mediawiki/You_Bastard) (a camel), and poorly designed silicon in the form of trolls, such as [Detritus](https://wiki.lspace.org/mediawiki/index.php/Detritus). Hex is a machine which thinks it is alive, is generally speaking a black box, runs on cheese and has an Anthill Inside. Only Ponder Stibbons really knows how it works, so I'm not going to burn my fingers on Hex. As far as You Bastard is concerned, a derivative would for sure make a funny name for the solver, except that insulting your academic advisors is probably not a good idea. That leaves Detritus. Unfortunately, trolls thermal throttle at room temperature. It also doesn't help that, in brutal honesty, Detritus is dumb for a troll. However, one thing a room-temperature Detritus has going for him is that he is the proud owner of the Piecemaker, a crossbow with a 2000lbs draw. As the L-Space wiki aptly describes the crossbow:

> Generally anything it is fired at is disintegrated.

The aim is to put Detritus and Piecemaker to work on DFVS instances, and so project DFVStritus was born.

*Maybe I should've hired an alchemist instead, they always manage to reduce (read: blow up) the Alchemist's guild to pebbles, and they work with solutions. You would figure that combining the two wouldn't be hard, well, not as hard as getting shot by Detritus.*