Simple Maturin/Rust example
===========================

## Creation

I used

```sh
    $ maturin new --bindings pyo3 sample
    $ cd sample
    $ ${EDITOR} src/lib.rs
```


## Testing

For completeness sake, you may want to open a Python repl to try importing the
`sample` package to see it's not yet available, until you `deploy` it with
maturin (see [Maturin documentation](https://www.maturin.rs/tutorial) for other commands/capabilities)

```sh
    $ python -c 'import sample; sample.greeting()'  # should fail import
    $ maturin deploy
    $ python -c 'import sample; sample.greeting()'  # should greet
```

