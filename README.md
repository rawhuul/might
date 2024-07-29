## Might

This tool executes REST API tests defined in a YAML-like format.

**Example Test Case:**

```yaml
Description: Test case for verifying the GET endpoint of the Example API
Author: John Doe
Method: GET
URL: https://google.com/
StatusCode: 400

Assertions:
  JSONPathExists: $.data
```

This test case will send a GET request to the specified URL, expect a 200 status code, and verify that the response contains a JSON object with a "data" property. 
