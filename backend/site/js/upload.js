import auth from './authentication.js';
// Initialize search results on page load
document.getElementById('uploadForm').addEventListener('submit', function (e) {
    e.preventDefault(); // Prevent the default form submission

    const user = auth.currentUser;
    if (user) {
        // User is signed in, get the ID token
        user.getIdToken(true).then(function (idToken) {
            // Set the token in the hidden input
            document.getElementById('firebaseToken').value = idToken;

            // Now submit the form
            e.target.submit();
        }).catch(function (error) {
            console.error('Error fetching ID token:', error);
            alert('Authentication error. Please sign in again.');
        });
    } else {
        // No user is signed in
        alert('You must be signed in to upload beatmaps.');
        // Optionally, redirect to sign-in page
    }
});