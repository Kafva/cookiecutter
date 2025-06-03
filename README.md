# cookiecutter
A CLI cookie manager for Firefox and Chromium.

![](https://i.imgur.com/aQZ7oMJ.png)

Currently does not support decryption of the `encrypted_value` field of
Chromium cookies.

## Examples
Interactive view over cookies from a specific file (screenshot)
```bash
cookiecutter --file cookies.sqlite tui
```
List all cookies from a given domain in a specific browser profile
```bash
cookiecutter --profile "Profile 1" cookies --domain github.com
```
List the domains that have cookies saved across all browsers
```bash
cookiecutter cookies --no-heading --fields Host|sort -u
```
List all fields for cookies from a specific domain
```bash
cookiecutter cookies --domain github.com --fields All
```
Remove all cookies except those from whitelisted domains across all browsers
```bash
cookiecutter clean --whitelist ~/.secret/cookie_whitelist --apply
```

