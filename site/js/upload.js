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

        if (formData.get('beatmap').size > 20000000) {
            feedback.classList.remove('d-none', 'alert-success');
            feedback.classList.add('alert-danger');
            feedback.textContent = 'ZIP size over 20MB limit.';
            return
        }

        const user = auth.currentUser;
        if (user) {
            user.getIdToken(true).then(async function (idToken) {
                formData.set('firebaseToken', idToken);
                const response = await fetch('/api/upload', {
                    method: 'POST',
                    body: formData
                });
                const result = await response.text();

                if (response.ok) {
                    uploadForm.reset();
                    window.location.href = 'search.html?' + result;
                } else {
                    feedback.classList.remove('d-none', 'alert-success');
                    feedback.classList.add('alert-danger');
                    feedback.textContent = result || 'An error occurred during upload.';
                }
            }).catch(function (error) {
                console.error('Error fetching ID token:', error);
                feedback.classList.remove('d-none', 'alert-success');
                feedback.classList.add('alert-danger');
                feedback.textContent = 'Error authenticating, please sign in again or ask for help in the discord.';
            });
        } else {
            feedback.classList.remove('d-none', 'alert-success');
            feedback.classList.add('alert-danger');
            feedback.textContent = 'You must sign in to upload a beatmap!';
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