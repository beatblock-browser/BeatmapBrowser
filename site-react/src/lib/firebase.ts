import { initializeApp } from "firebase/app";
import { getAuth, setPersistence, browserLocalPersistence } from "firebase/auth";

const firebaseConfig = {
    apiKey: "AIzaSyDIEQBCB65cEolBwKkPnAi74Ja5bFiav3s",
    authDomain: "beatblockbrowser.firebaseapp.com",
    projectId: "beatblockbrowser",
    storageBucket: "beatblockbrowser.appspot.com",
    messagingSenderId: "477037278423",
    appId: "1:477037278423:web:8bd41df2941f65e3162c92",
    measurementId: "G-W3N8R9EVJ9"
};

export const app = initializeApp(firebaseConfig);
export const auth = getAuth(app);
// Ensure the session persists across tabs/reloads without re-prompting
setPersistence(auth, browserLocalPersistence).catch(() => {
  // Non-fatal; apiFetchAuth will still refresh on demand
});

