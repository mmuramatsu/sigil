# Sigil

## Overview

Sigil is a command-line tool written in Rust that verifies the integrity of a file by comparing its actual file type with its declared file type (based on the file extension). It identifies the actual file type by reading the file's "magic numbers" (the first few bytes of the file) and comparing them against a database of known file signatures.

This tool is useful for detecting files that have been misnamed or intentionally disguised with a misleading file extension.

## How It Works

Many file formats have a unique sequence of bytes at the beginning of the file, known as a "magic number" or "file signature." Sigil uses these signatures to determine the true type of a file.

The process is as follows:

1.  **Load Signatures:** Sigil reads a JSON file containing a list of known file signatures. Each entry in the JSON file includes the file type, the signature (as a sequence of bytes), and the offset at which the signature is expected to be found.
2.  **Build Trie:** The signatures are loaded into a [Trie](https://en.wikipedia.org/wiki/Trie) data structure. A Trie is used for efficient searching of the magic numbers.
3.  **Read File Header:** Sigil reads the first few bytes of the input file. The number of bytes read is determined by the maximum possible length of a signature and its offset, as defined in the JSON file.
4.  **Search and Compare:** The Trie is used to search for a matching signature in the file's header. If a match is found, the identified file type is compared with the file's extension.
5.  **Report Result:** Sigil reports whether the file's declared type matches its actual type.

## Usage

To use Sigil, you need to have the Rust toolchain installed.

1.  **Clone the repository:**
    ```sh
    git clone https://github.com/mmuramatsu/sigil.git
    cd sigil
    ```

2.  **Compile the project:**
    ```sh
    cargo build --release
    ```

3.  **Run the program:**
    ```sh
    ./target/release/sigil <FILE_PATH>
    ```
    Replace `<FILE_PATH>` with the path to the file you want to check.

    You can also provide a custom JSON file with file signatures using the `-i` or `--input-json-file` flag:
    ```sh
    ./target/release/sigil <FILE_PATH> -i /path/to/your/signatures.json
    ```
    If no input file is provided, Sigil will look for `data/magic_numbers_reference.json`.

## JSON Format

The JSON file for file signatures should be an array of objects, where each object has the following format:

```json
[
  {
    "type": "PNG",
    "offset": 0,
    "signature": [137, 80, 78, 71, 13, 10, 26, 10]
  },
  {
    "type": "JPG",
    "offset": 0,
    "signature": [255, 216, 255]
  }
]
```

*   `type`: The file type extension (e.g., "PNG", "JPG").
*   `offset`: The position in the file (in bytes) where the signature begins.
*   `signature`: An array of bytes representing the magic number.

## Contributing

Contributions are welcome! If you have any ideas, suggestions, or bug reports, please open an issue or submit a pull request. For major changes, please open an issue first to discuss what you would like to change.

## Contact

If you have any questions, suggestions, or just want to connect, feel free to reach out:

*   **Webpage:** [mmuramatsu.com](https://mmuramatsu.com/)
*   **GitHub:** [@mmuramatsu](https://github.com/mmuramatsu)
*   **Email:** [junior_muramatsu@hotmail.com](mailto:junior_muramatsu@hotmail.com)
*   **LinkedIn:** [Mario Muramatsu Júnior](https://www.linkedin.com/in/mario-muramatsu-jr/)

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
