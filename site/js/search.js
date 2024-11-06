// Function to handle upvoting
import {runLoggedIn, showError} from "./authentication.js";

async function handleUpvote(event, id) {
    const button = event.currentTarget;
    const countSpan = button.querySelector('.upvote-count');
    let count = parseInt(countSpan.textContent, 10);
    count += 1;
    countSpan.textContent = count;

    await runLoggedIn(async function (idToken) {
        const response = await fetch('/api/upvote', {
            method: 'POST',
            body: JSON.stringify({
                'firebaseToken': idToken,
                'mapId': id
            })
        });
        const result = await response.text();

        if (response.ok) {
            // Optional: Change button appearance after upvote
            button.classList.remove('btn-outline-light');
            button.classList.add('btn-success');
            button.innerHTML = `<span class="bi bi-hand-thumbs-up-fill"></span> ${count}`;
        } else {
            console.error('Error upvoting: ', response)
            showError(result || 'An error occurred when upvoting.');
        }
    });
    // Disable the button to prevent multiple upvotes
    button.disabled = true;
}

// Function to display search results
async function displaySearchResults() {
    const resultsContainer = document.getElementById('search-results');
    const noResultsContainer = document.getElementById('no-results');
    const template = document.getElementById('search-result-template');

    try {
        // Show a loading indicator (optional)
        resultsContainer.innerHTML = '<p>Loading results...</p>';

        // Initialize the resolver outside the functions to make it accessible to both
        let resolveSignal;

        // Create a Promise that Function A will await
        const signalPromise = new Promise((resolve) => {
            resolveSignal = resolve;
        });

        let upvoted_list = runLoggedIn(async function (idToken) {
            let upvoted = await fetch(`/api/upvote_list`, {
                method: 'POST',
                body: JSON.stringify({
                    'firebaseToken': idToken,
                })
            });
            await signalPromise;

            let json = await upvoted.json();
            for (let i = 0; i < json.length; i++) {
                let element = json[i];
                let map = document.querySelector(`.map-${element.id['String']}`);
                if (map != null) {
                    let button = map.querySelector('.upvote-button');
                    button.classList.remove('btn-outline-light');
                    button.classList.add('btn-success');
                    button.innerHTML = `<span class="bi bi-hand-thumbs-up-fill"></span> ${button.textContent}`
                }
            }
        }, true)

        // Fetch data from the /api/search endpoint
        let response;
        try {
            response = await fetch(`/api/search${window.location.search}`, {
                method: 'GET',
                headers: {
                    'Accept': 'application/json'
                }
            });
        } catch (e) {
            console.log(e);
        }

        if (response != null && response.status === 429) {
            resultsContainer.innerHTML = '';
            showError('Please stop spamming search requests!');
            return
        }

        if (response == null || !response.ok) {
            resultsContainer.innerHTML = '';
            showError('Failed to search, see console log');
            return
        }

        const searchResult = await response.json();
        document.getElementById('search-query').textContent = searchResult.query;
        const beatMaps = searchResult.results;

        // Clear previous results
        resultsContainer.innerHTML = '';

        if (beatMaps.length > 0) {
            beatMaps.forEach(map => {
                // Clone the template
                const clone = template.content.cloneNode(true);
                clone.querySelector('.card').classList.add("map-" + map.id.id['String']);
                // Populate the clone with actual data
                clone.querySelector('.card-title').textContent = map.song;
                clone.querySelectorAll('.card-text')[0].textContent = map.artist;
                clone.querySelectorAll('.card-text')[1].textContent = map.charter;
                clone.querySelectorAll('.card-text')[2].textContent = map.difficulties.map(d => d.display).join(", ");
                if (map.image != null) {
                    clone.querySelector('img').src = 'output/' + map.image;
                } else {
                    clone.querySelector('img').src = 'beatblocks.jpg';
                }
                clone.querySelector('a.btn').href = 'output/' + map.download;
                const button = clone.querySelector('.upvote-button');
                button.querySelector('.upvote-count').textContent = map.upvotes;
                button.addEventListener('click', (event) => handleUpvote(event, map.id));

                // Append the clone to the results container
                resultsContainer.appendChild(clone);
            });
        } else {
            // No results found
            noResultsContainer.classList.remove('invisible');
            resultsContainer.innerHTML = '';
        }

        resolveSignal()
        await upvoted_list;
    } catch (error) {
        console.error('Error fetching search results:', error);
        resultsContainer.innerHTML = '';
        showError('An error occurred while fetching search results. Please try again later.');
    }
}

// Initialize search results on page load
document.addEventListener('FinishInline', async function () {
    await displaySearchResults();
});