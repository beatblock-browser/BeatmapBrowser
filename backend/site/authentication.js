// Import the functions you need from the SDKs you need
import { initializeApp } from "deps/firebase-app.js";
import { getAuth, signInWithPopup, GoogleAuthProvider, onAuthStateChanged } from "deps/firebase-auth.js";

// Your web app's Firebase configuration
// For Firebase JS SDK v7.20.0 and later, measurementId is optional
const firebaseConfig = {
    apiKey: "AIzaSyDIEQBCB65cEolBwKkPnAi74Ja5bFiav3s",
    authDomain: "beatblockbrowser.firebaseapp.com",
    projectId: "beatblockbrowser",
    storageBucket: "beatblockbrowser.appspot.com",
    messagingSenderId: "477037278423",
    appId: "1:477037278423:web:8bd41df2941f65e3162c92",
    measurementId: "G-W3N8R9EVJ9"
};

// Initialize Firebase
const app = initializeApp(firebaseConfig);
const auth = getAuth(app);
export default auth;

// Google Sign-In Button Event Listener
const googleSignInBtn = document.getElementById('googleSignInBtn');
const errorBox = document.getElementById('error-message');

if (googleSignInBtn != null) {
    googleSignInBtn.addEventListener('click', () => {
        const provider = new GoogleAuthProvider();

        signInWithPopup(auth, provider)
            .then((result) => {
                // The signed-in user info
                const user = result.user;

                console.log('User signed in:', user);

                // Redirect to home page or dashboard
                window.location.href = 'index.html';
            })
            .catch((error) => {
                // Handle Errors here.
                const errorMessage = error.message;

                console.error('Error during sign in:', error);

                // Display error message to the user
                if (errorMessage) {
                    errorBox.textContent = `Error: ${errorMessage}`;
                    errorBox.style.display = 'block';
                }
            });
    });
}

// Authentication State Listener
onAuthStateChanged(auth, (user) => {
    const loginNavLink = document.getElementById('loginNavLink');
    const uploadNavLink = document.getElementById('uploadNavLink');
    const accountNavLink = document.getElementById('accountNavLink');
    if (user) {
        console.log('User is signed in:', user);
        // Hide "Log In" link
        loginNavLink.classList.add('d-none'); // Using Bootstrap's d-none class
        // Show "Upload" and "Account" links
        uploadNavLink.classList.remove('d-none');
        accountNavLink.classList.remove('d-none');
    } else {
        console.log('No user is signed in.');
        // Show "Log In" link
        loginNavLink.classList.remove('d-none');
        // Hide "Upload" and "Account" links
        uploadNavLink.classList.add('d-none');
        accountNavLink.classList.add('d-none');
    }
});