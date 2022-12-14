# 0.5.0

support substrate 3.0.0. tested with paritytech/substrate@743accbe3256de2fc615adcaa3ab03ebdbbb4dbd

depend on substrate git without pinning a revision. use `cargo` to select a revision:

```
# may need to do twice!
cargo update -p sp-std --precise 743accbe3256de2fc615adcaa3ab03ebdbbb4dbd
cargo update -p sp-std --precise 743accbe3256de2fc615adcaa3ab03ebdbbb4dbd
```