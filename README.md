# May

May is application for testing APIs. It allows you to send HTTP requests (GET and POST) to a specified URL and receive JSON responses. The app uses the `reqwest` crate for making HTTP requests and the `serde_json` crate for handling JSON data.

## Usage

1. Enter the desired HTTP request in the format of `METHOD URL`. For example, to send a GET request to `https://api.example.com/endpoint`, type: `GET https://api.example.com/endpoint`.

2. For POST requests, the app will prompt for a JSON body. Enter the JSON body to include in the request.

3. The app will display the response in a pretty-printed JSON format.

4. To exit the app, type `exit`.

## Features

### Upcoming Features

The following features are planned for future updates:

- Syntax highlighting: Enhance the user interface with syntax highlighting for better readability.
- Up/down cursor with history: Allow navigation through command history using the up and down arrow keys.
- Session history support: Maintain a history of API requests and responses for the current session.
- Addition of PUT, PATCH, and DELETE methods: Extend support for additional HTTP methods.
- Session management: Enable session management to save and load previous sessions for easy recall.

## Requirements

- Rust (I'm using `1.71.0-nightly`)
- `reqwest` crate
- `serde_json` crate
- `rustyline` crate

## Screenshot

![Screenshot](/screenshots/screenshot1.png?raw=true "Screenshot")

## Installation

1. Install Rust by following the instructions at [rustup.rs](https://rustup.rs/).

2. Clone this repository:

   ```bash
   git clone https://github.com/basicfunc/may.git
   ```

3. Change to the project directory:

   ```bash
   cd may
   ```

4. Build and run the application:

   ```bash
   cargo run
   ```

## Contributions

Contributions to this project are welcome! Feel free to open issues or submit pull requests with improvements, bug fixes, or new features.

## License

This project is licensed under the [MIT License](LICENSE). Feel free to modify and enhance the project as needed.