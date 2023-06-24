# copier

Copier is a small command line tool to copy files from
one directory to another with an ignore file. For example,
you can use it to back up a directory without gigantic 
`node_modules` or `target`s clogging up the target.

Usage:
```
copier <source> <target> <ignorelist>
```
where `<ignorelist>` is a file of the syntax:
```
# Comment

# Directory containing `file.txt`
/file.txt

# Directory name
node_modules
```

