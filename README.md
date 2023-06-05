# May

A simple command-line application that allows users to send HTTP requests and view the response, making it convenient for testing APIs. The tool supports various HTTP methods like `GET`, `POST`, `PUT`, `PATCH`, and `DELETE`. It also provides the option to pretty-print the response in JSON format or display it as a table.

## Features

1. Send HTTP requests:
   - `GET`: Retrieve data from a specified URL.
   - `POST`: Send data to a specified URL.
   - `PUT`: Update data at a specified URL.
   - `PATCH`: Partially update data at a specified URL.
   - `DELETE`: Delete data at a specified URL.

2. Interactive REPL (Read-Eval-Print Loop):
   - Users can interactively enter commands in the tool.
   - Commands are executed and responses are displayed in real-time.

3. History:
   - The tool keeps a history of previous requests and responses.
   - Users can view the session history to see the details of past requests and responses.

4. Pretty-print response:
   - The tool provides the option to pretty-print the JSON response.
   - Users can choose between a table format or a formatted JSON string.

5. Error handling:
   - The tool provides error handling for various scenarios, such as invalid methods or URLs, failed requests, etc.

6. History persistence:
   - The tool saves the session history to a file (`history.txt`) and loads it on startup.
   - This ensures that the history is preserved between different tool sessions.

## Usage

1. Start the tool.
2. Enter commands in the following format: `<HTTP_METHOD> <URL>`.
   - For example: `GET https://api.example.com/data`.
3. View the response.
   - If the response is JSON, it can be displayed as a table or a formatted string.
   - The session history can be accessed with the command `history`.
4. Continue entering commands or type `exit` to exit the tool.

## Command-line Arguments

The tool supports the following command-line arguments:

- `-j` or `--json`: Outputs the response in JSON format.

Example usage: `may -j`

## Dependencies

The tool relies on the following external crates:

- `argh`: Command-line argument parsing.
- `json_to_table`: Converts JSON data to a table format.
- `reqwest`: HTTP client for sending requests.
- `rustyline`: Library for creating an interactive command-line REPL.
- `serde_json`: JSON serialization and deserialization.
- `tabled`: Formats tabular data.

## Credits

This tool was developed using the Rust programming language and various open-source crates.

## License

This tool is licensed under the [MIT License](LICENSE).