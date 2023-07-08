# Contribution guidelines
Thank you for considering contributing to the IPlan project!

## Translation
Let's get started with where files are located and what they are for.

Localization-related files are inside [po](https://github.com/iman-salmani/iplan/tree/1e0f0081acebff48db156300498e86ca437b2cef/po) directory.
The directory should be something like this:
```
po
- LINGUAS
- iplan.pot
- fa.po
- fr.po
...
```

* **LINGUAS** contains available languages represented by their code
* **iplan.pot** is a sample for adding a new language
* Files with `.po` extension contains translation texts

As I mentioned, translation files use po file format. if you are not familiar with that there are plenty of articles about how to edit them.

If you wanna add a new language follow this:
1. Duplicated the `iplan.pot` file and rename it to `language_code.po`
2. Add language code to `LINGUAS` file