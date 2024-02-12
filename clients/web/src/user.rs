// This is the Yonder User library

// Fetch Yonder User by e-mail address.
// NOTE:
// * This e-mail address is provided by the Auth Provider. It cannot be specified by the user!
// * We need a valid access token -- the User service will check for an expired Access token and return a 401
// * We should have a request helper that autochecks the access token expiration and uses the refresh token before the access token expires and after receiving a 401, if necessary
// * We should have a generic decorator that requires a valid access token (syntactic sugar) -- a Rocket validation?
