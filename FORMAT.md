```
[TOOL_NAME: str(max: 64)]
[DETECTION_METHOD::ID][PARAMETERS]
[CLEANING_METHOD::ID][PARAMETERS]
[SIZE_DIR: Path]
```

<sub>**Note:** Multiple tools are separated with newlines.</sub><br>
<sub>**Note:** Parameters are separated with semicolons (;)</sub>

## Detection Methods
* **Binary Exists**
    * **ID:** 1
    * **Arguments:** `[BINARY_NAME: str]`
    * **Description:** Checks if a binary exists on your PATH.

* **Environment Variable**
    * **ID:** 2
    * **Arguments:** `[VAR_NAME: str]`
    * **Description:** Checks if an environment variable is present. (Does not check equality)

* **Path Exists**
    * **ID:** 3
    * **Arguments:** `[PATH: Path]`
    * **Description:** Checks if a directory or file exists.

## Cleaning Methods
* **Run Command**
    * **ID:** 1
    * **Arguments:** `[EXE: str]` `[ARGS: str[] separated with spaces]`
    * **Description:** Runs a command in the terminal.

* **Clean Path**
    * **ID:** 2
    * **Arguments:** `[PATH: Path]`
    * **Description:** Deletes everything in the folder without deleting the folder.

## Path
Paths are defined just like the way you write directories in your os with some differences:
  - Supports both slashes (/) and backslashes (\\).
  - Environment variables are NOT supported.
  - Expanding home (~) IS supported.