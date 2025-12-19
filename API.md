# API Documentation

This document provides an overview of the API endpoints available in the QSO Logger application. Each endpoint is described with its method, URL, parameters, and expected responses.

## Base URL

The base URL for all API endpoints is:

```https://api.qso-logger.com/v1```

## Endpoints

### 1. Login

- **Method:** POST
- **URL:** `/auth/login`
- **Parameters:**
  - `username` (string, required): The username of the user.
  - `password` (string, required): The password of the user.
- **Response:**
  - `200 OK`: Returns a JSON object with an authentication token.
  - `401 Unauthorized`: Invalid username or password.

### 2. Register

- **Method:** POST
- **URL:** `/auth/register`
- **Parameters:**
  - `username` (string, required): The desired username.
  - `email` (string, required): The user's email address.
  - `password` (string, required): The desired password.
- **Response:**
  - `201 Created`: Returns a JSON object with user details.
  - `400 Bad Request`: Missing or invalid parameters.

### 3. Get QSO Logs

- **Method:** GET
- **URL:** `/logs`
- **Parameters:**
  - `auth_token` (string, required): The authentication token obtained from the login endpoint.
  - `start_date` (string, optional): The start date for filtering logs (format: YYYY-MM-DD).
  - `end_date` (string, optional): The end date for filtering logs (format: YYYY-MM-DD).
- **Response:**
  - `200 OK`: Returns a JSON array of QSO log entries.
  - `401 Unauthorized`: Invalid or missing authentication token.

### 4. Add QSO Log

- **Method:** POST
- **URL:** `/logs`
- **Parameters:**
  - `auth_token` (string, required): The authentication token.
  - `callsign` (string, required): The callsign of the contact.
  - `frequency` (float, required): The frequency of the QSO in MHz.
  - `mode` (string, required): The mode of communication (e.g., SSB, CW).
  - `date` (string, required): The date of the QSO (format: YYYY-MM-DD).
  - `time` (string, required): The time of the QSO (format: HH:MM:SS).
- **Response:**
  - `201 Created`: Returns a JSON object with the created QSO log entry.
  - `400 Bad Request`: Missing or invalid parameters.
  - `401 Unauthorized`: Invalid or missing authentication token.

### 5. Delete QSO Log

- **Method:** DELETE
- **URL:** `/logs/{log_id}`
- **Parameters:**
  - `auth_token` (string, required): The authentication token.
  - `log_id` (integer, required): The ID of the QSO log entry to delete.
- **Response:**
  - `204 No Content`: Successfully deleted the QSO log entry.
  - `401 Unauthorized`: Invalid or missing authentication token.
  - `404 Not Found`: QSO log entry not found.

### 6. Update QSO Log

- **Method:** PUT
- **URL:** `/logs/{log_id}`
- **Parameters:**
  - `auth_token` (string, required): The authentication token.
  - `log_id` (integer, required): The ID of the QSO log entry to update.
  - `callsign` (string, optional): The updated callsign of the contact.
  - `frequency` (float, optional): The updated frequency of the QSO in MHz.
  - `mode` (string, optional): The updated mode of communication (e.g., SSB, CW).
  - `date` (string, optional): The updated date of the QSO (format: YYYY-MM-DD).
  - `time` (string, optional): The updated time of the QSO (format: HH:MM:SS).
- **Response:**
  - `200 OK`: Returns a JSON object with the updated QSO log entry.
  - `400 Bad Request`: Missing or invalid parameters.
  - `401 Unauthorized`: Invalid or missing authentication token.
  - `404 Not Found`: QSO log entry not found.

### 7. Get User Profile

- **Method:** GET
- **URL:** `/user/profile`
- **Parameters:**
  - `auth_token` (string, required): The authentication token.
- **Response:**
  - `200 OK`: Returns a JSON object with user profile information.
  - `401 Unauthorized`: Invalid or missing authentication token.

### 8. Update User Profile

- **Method:** PUT
- **URL:** `/user/profile`
- **Parameters:**
  - `auth_token` (string, required): The authentication token.
  - `email` (string, optional): The updated email address.
  - `password` (string, optional): The updated password.
- **Response:**
  - `200 OK`: Returns a JSON object with the updated user profile information.
  - `400 Bad Request`: Missing or invalid parameters.
  - `401 Unauthorized`: Invalid or missing authentication token.

## Error Codes

- `400 Bad Request`: The request could not be understood or was missing required parameters.
- `401 Unauthorized`: Authentication failed or user does not have permissions for the requested operation.
- `403 Forbidden`: Access to the requested resource is forbidden.
- `404 Not Found`: The requested resource could not be found.

