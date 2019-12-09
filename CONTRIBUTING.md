# Contributing
Contributions are welcome! If you need to discuss the change you wish to make feel free to open an issue. We're also on [Gitter]().

**Working on your first Pull Request?** You can learn how from this *free* series [How to Contribute to an Open Source Project on GitHub](https://egghead.io/series/how-to-contribute-to-an-open-source-project-on-github)

## Contribute code to SODA Test Service
### Workflow
We only work with the `master` branch for deployments and with feature branches (ex : `feature/my-feature`) for new features. You can also create a fix branch (ex : `fix/my-fix`) when you want to make a patch.

1. Fork the repository
2. Create a new feature / fix branch from the latest version of master
2. Start coding
3. Open a pull request, a core developer will review your contribution and merge it

### Coding Style
We follow the [Rust Style Guide](https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md).

1. Install clippy and rustfmt

```bash
rustup component add clippy
rustup component add rustfmt
```

2. Run clippy and fix the warnings

```bash
cargo clippy
```

This command shouldn't return anything if your code is valid. Else, fix your code.

3. Run rustfmt

To see changes that need to be made :
```bash
cargo fmt --all -- --check
```

If your code is well formatted, you shouldn't see any error or output. Else run the following command to apply the formatting :

```bash
cargo fmt --all
```

This command will not produce any output but your files will be corrected.

## Issues and labels
- Good first issue : if you're searching to learn and contribute, issues with the label `good first issue` are for you!
- Mentoring : someone will provide you mentoring for an issue.
- Easy / medium / hard  : these labels help you to know the estimated complexity of issues.
- Help wanted : someone is looking for some help.

This contribution guide is inspired from the [Diesel project](https://github.com/diesel-rs/diesel/blob/master/CONTRIBUTING.md).