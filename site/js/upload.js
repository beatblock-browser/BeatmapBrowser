import auth from './authentication.js';
// Initialize search results on page load
document.getElementById('uploadForm').addEventListener('submit', async function (e) {
    const uploadForm = document.getElementById('uploadForm');
    const feedback = document.getElementById('feedback');
    const submitButton = document.getElementById('submitButton');
    e.preventDefault();

    feedback.classList.add('d-none');
    feedback.classList.remove('alert-success', 'alert-danger');
    feedback.textContent = '';

    submitButton.disabled = true;
    submitButton.textContent = 'Uploading...';

    try {
        const formData = new FormData(uploadForm);

        const user = auth.currentUser;
        if (user) {
            user.getIdToken(true).then(function (idToken) {
                formData.set('firebaseToken', idToken);
            }).catch(function (error) {
                console.error('Error fetching ID token:', error);
                feedback.classList.remove('d-none', 'alert-success');
                feedback.classList.add('alert-danger');
                feedback.textContent = 'Error authenticating, please sign in again or ask for help in the discord.';
            });
            if (formData.get('firebaseToken') == null) {
                return
            }
        } else {
            feedback.classList.remove('d-none', 'alert-success');
            feedback.classList.add('alert-danger');
            feedback.textContent = 'You must sign in to upload a beatmap!';
            return;
        }

        const response = await fetch('/api/upload', {
            method: 'POST',
            body: formData
        });

        const result = await response.json();

        if (response.ok) {
            feedback.classList.remove('d-none', 'alert-danger');
            feedback.classList.add('alert-success');
            feedback.textContent = result.message || 'BeatMap uploaded successfully!';
            uploadForm.reset();
        } else {
            feedback.classList.remove('d-none', 'alert-success');
            feedback.classList.add('alert-danger');
            feedback.textContent = result.error || 'An error occurred during upload.';
        }
    } catch (error) {
        // Network or other errors
        console.error('Upload error:', error);
        feedback.classList.remove('d-none', 'alert-success');
        feedback.classList.add('alert-danger');
        feedback.textContent = 'Failed to upload BeatMap. Please try again later.';
    } finally {
        // Re-enable the submit button
        submitButton.disabled = false;
        submitButton.textContent = 'Upload BeatMap';
    }
});