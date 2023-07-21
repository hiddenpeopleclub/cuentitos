# Command Line Interface

## Compile

To compile a story in `cuentitos`, you must prepare two specific files: a script file written in `palabritas`, and a configuration file. Once these files are set up correctly, you can compile your story with the command provided below:

```
compile {path to script file} {destination file name} 
```

## Run

Once your story is compiled, you can run it using the following command:

```
run {path to cuentitos file}
```

### Commands
Once in `run` mode, you have the following commands available:

- `Enter`: progresses the story
- `?`: displays the current state of cuentitos
- `variables`: shows the status of the palabritas variables
- `sections`: lists the palabritas sections
- `q`: closes the program
- `set {variable} {value}`: works the same as the `set` command in `palabritas`. It's used to change the value of variables during runtime.
- `-> {section}`: works the same as the `divert` command in `palabritas`. It's used to jump to a particular section.
- `<-> {section}`: works the same as the `boomerang divert` in `palabritas`. It's used to jump to a particular section, but after finishing that path, it returns to the point before the `boomerang divert`. 
- `reset story`: resets the story back to the beginning.
- `reset state`: resets the state of cuentitos back to default.
- `reset` / `reset all`: resets both the story and the state of cuentitos to default.
- `h` / `help`: prints all of the available commands with a brief desciption.


### Filters

The commands `variables`, `sections`, and `?`, can be refined using space-separated filter terms. Here's an illustrative code example:

When you type `variables`, you get:

``` 
>variables
Variables:
vida: 10
velocidad: 1
```

When you apply a filter, such as `vid`, it narrows down the results:

``` 
>variables vid
vida: 10
``` 

In the latter example, only variables that contain the string `vid` are displayed. 

## Help

You can use the `help` command to see all the available commands for the cli.

## Watch

You can use the `watch` command to compile a story and run it aftewards. This command also watches for changes in the script directory so if the script or the configuration files change, the cli will prompt you for confirmation in case you want to recompile or restart the story.

```
watch {path to script file}
```
