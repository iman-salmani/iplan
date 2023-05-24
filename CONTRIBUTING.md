# Contribution guidelines
Thank you for considering contributing to the IPlan project!

## Translation
First of all, Check if your language already exists. You can find po files in the po folder which are named by languages code.
1. Clone repository
```bash
git clone https://github.com/iman-salmani/iplan.git
```
2. Build project
```bash
meson setup _build .
meson compile -C _build
```
3. Generate the pot file.
```bash
cd _build
meson compile iplan-pot
```
4. Rename the iplan.pot file (you can find it on po folder) to your language code like 'en.po' (remember to change file extension ).
5. Also, add your language code to the end of the LINGUAS file.
