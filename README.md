# May

A simple command-line application that allows users to send HTTP requests and view the response, making it convenient for testing APIs. It supports various HTTP methods like `GET`, `POST`, `PUT`, `PATCH`, and `DELETE`. It also provides the option to pretty-print the response in JSON format or display it as a table.

## Features

1. Send HTTP requests:
   - `GET`: Retrieve data from a specified URL.
   - `POST`: Send data to a specified URL.
   - `PUT`: Update data at a specified URL.
   - `PATCH`: Partially update data at a specified URL.
   - `DELETE`: Delete data at a specified URL.

2. Interactive REPL (Read-Eval-Print Loop):
   - Users can interactively enter commands in the app.
   - Commands are executed and responses are displayed in real-time.

3. History:
   - It keeps a history of previous requests and responses.
   - Users can view the session history to see the details of past requests and responses.

4. Pretty-print response:
   - It provides the option to pretty-print the JSON response.
   - Users can choose between a table format or a formatted JSON string.

5. Error handling:
   - It does error handling for various scenarios, such as invalid methods or URLs, failed requests, etc.

6. History persistence:
   - It also saves the session history to a file (`history.txt`) and loads it on startup.
   - This ensures that the history is preserved between different sessions.

## Upcoming Features

- [ ] Request Headers: Allow users to specify custom headers for their requests.
- [ ] Authentication: Support different authentication methods like Basic Authentication, API keys, OAuth, etc.
- [x] Request Timeout: Allow users to set a timeout for their requests.
- [x] Response Status Codes: Display the HTTP status code along with the response. This will provide more context about the success or failure of the request.
- [x] Response Time: Show the time taken to receive the response from the server.
- [ ] Environment Variables: Support the use of environment variables, allowing users to store and reference variables like API keys or base URLs without hardcoding them in commands.
- [ ] Batch Requests: Allow users to send multiple requests in a batch, either by reading a file containing a list of requests or by providing a formatted input.
- [ ] File Upload: Enable users to upload files as part of their requests, such as sending images, documents, or other binary data to the server.
- [ ] Response Caching: Implement a caching mechanism to store responses locally. This can help speed up subsequent requests to the same endpoint, especially for APIs with frequently accessed data.

## Usage

1. Start the application.
2. Enter commands in the following format: `<HTTP_METHOD> <URL>`.
   - For example: `GET https://api.example.com/data`.
3. View the response.
   - If the response is JSON, it can be displayed as a table or a formatted string.
   - The session history can be accessed with the command `history`.
4. Continue entering commands or type `exit` to exit the app.

## Command-line Arguments

It supports the following command-line arguments:

- `-j` or `--json`: Outputs the response in JSON format.

Example usage: `may -j`

## Dependencies

It relies on the following external crates:

- `argh`: Command-line argument parsing.
- `json_to_table`: Converts JSON data to a table format.
- `reqwest`: HTTP client for sending requests.
- `rustyline`: Library for creating an interactive command-line REPL.
- `serde_json`: JSON serialization and deserialization.
- `tabled`: Formats tabular data.

## Credits

It was developed using the Rust programming language and various open-source crates.

## License

It is licensed under the [MIT License](LICENSE).