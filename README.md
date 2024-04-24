Dockerization of a Rust/Pyo3/Maturin experiment setup
=====================================================

🚧 Work in progress 🚧

RUst - MAturin - pyO3: rumao3
-----------------------------

The goal here is just to create a trivial playground for mucking about with Pyo3
and Maturin as a means to build Python extensions using Rust.

Start by creating a project for yourself and deploying it.


# Quick start/Demo

To run using the existing docker image and see what the container is *for*:

```sh
docker run --rm -it -v ${PWD}:/src kfsone/rumao3
cd sample 
python -c 'import sample; sample.greeting()'  # <- import fail
maturin develop  # <- build and deploy the rust extension into the virtualenv
python -c 'import sample; sample.greeting()'  # <- success
python -c 'import sample; text = sample.tac("Cargo.toml"); print(text)'  # <- prints the README.md in reverse
exit
```

*NOTE* If you are using the Docker image rather than the github repos, a copy of the sample
is baked into the image at /opt/sample, but be aware that any changes to this will be lost
on container restart.


# Using the package

## Start the container

```sh
docker run --rm -it -v ${PWD}:/src kfsone/rumao3
```

--rm
    Delete container after you're done
-it
    Interative, terminal
-v ${PWD}:/src
    Mount the *current* directory as /src
kfsone/rumao3
    What I call the package; if you rebuild this image from source, use that name instead, duh.


## Create and build

Inside the container:

```sh
maturin new --bindings pyo3 mypackage
```

This will initialize a new Rust crate and pyo3 package combo in the directory 'mypackage'
which will also be visible in the directory you launched from back on the host machine.

Back in the container:
```sh
cd mypackage
nano src/lib.rs
```

(nano, vim and neovim are installed) or edit from your favorite IDE/editor out on the host
machine.

Once you're done, time to build and deploy and try your package inside the container:

```sh
maturin deploy
python
import mypackage
help(mypackage)
```


