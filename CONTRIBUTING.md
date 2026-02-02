# Contributing to Nil

First off, thank you for considering contributing to Nil! It's people like you who make it a better tool for everyone to use.

## How Can I Help?

### 1. Adding New Rules (Easiest)
The most helpful thing you can do is expand our `rules.txt` templates. If you know where a specific app stores its cache, please share it with us.

**How to suggest a rule:**
1. Check the `templates/` folder for your OS to see if the rule exists already.
2. Open an Issue or a Pull Request with the new block using the format specified in [FORMAT.md](FORMAT.md).

## 2. Reporting Bugs
If Nil crashed or didn't clean a folder it was supposed to:
- Open an Issue.
- Include your OS and the version of Nil you are using.
- Provide the `rules.txt` entry that caused the issue.

### 3. Code Contributions
If you want to add a new detection method, cleaning method or just another feature you can follow these steps:
1. Fork the repository.
2. Create a new branch for the feature. (`git checkout -b feature/awesome-feature`)
3. Make sure your project is formatted via `cargo fmt`
4. Test your code to ensure it works as expected (If possible test it on other operating systems too!).
5. Commit and push. Please keep your commit messages brief and clear.
6. Sync your fork with the original repository to avoid merge conflicts.
7. Open a pull request. In your pull request's description please include:
- What changed?
- Why did it change?