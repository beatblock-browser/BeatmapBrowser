// Function to handle upvoting
import {runLoggedIn, showError} from "./authentication.js";
import {downloadMap, removeMap} from "./oneclick_communicator.js";

// Function to display search results
async function displaySearchResults() {
    const resultsContainer = document.getElementById('search-results');
    const noResultsContainer = document.getElementById('no-results');
    const template = document.getElementById('search-result-template');

    try {
        // Show a loading indicator (optional)
        resultsContainer.innerHTML = '<p>Loading results...</p>';

        // Initialize the resolver outside the functions to make it accessible to both
        let finishSearchResolve;

        // Create a Promise that Function A will await
        const finishedSearch = new Promise((resolve) => {
            finishSearchResolve = resolve;
        });

        let upvoted_list = runLoggedIn((id) => updateSearchPage(id, finishedSearch), () => {})

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
                makeDownloadButton(clone.querySelector('.oneclick'), map.id.id['String']);
                clone.querySelector('.oneclick').disabled = true;

                clone.querySelector('.upvote-count').textContent = map.upvotes;
                makeUpvoteButton(clone.querySelector('.upvote-button'), map.id.id['String']);
                clone.querySelector('.upvote-button').disabled = true;

                // Append the clone to the results container
                resultsContainer.appendChild(clone);
            });
        } else {
            // No results found
            noResultsContainer.classList.remove('invisible');
            resultsContainer.innerHTML = '';
        }

        finishSearchResolve()
        await upvoted_list;
    } catch (error) {
        console.error('Error fetching search results:', error);
        resultsContainer.innerHTML = '';
        showError('An error occurred while fetching search results. Please try again later.');
    }
}

async function updateSearchPage(idToken, finishedSearch) {
    {
        let upvoted = await fetch(`/api/account_data`, {
            method: 'POST',
            body: JSON.stringify({
                'firebaseToken': idToken,
            })
        });
        await finishedSearch;

        let json = await upvoted.json();
        let upvotes = json.upvoted;
        for (let i = 0; i < upvotes.length; i++) {
            let element = upvotes[i];
            let map = document.querySelector(`.map-${element.id['String']}`);
            if (map != null) {
                makeUnvoteButton(map.querySelector('.upvote-button'), element);
            }
        }

        let downloaded = json.downloaded;
        for (let i = 0; i < downloaded.length; i++) {
            let element = downloaded[i];
            let map = document.querySelector(`.map-${element.id['String']}`);
            if (map != null) {
                makeRemoveButton(map.querySelector('.oneclick'), element.id['String']);
            }
        }

        document.querySelectorAll('.slow-loader').forEach((element) => element.disabled = false);
    }
}

// Initialize search results on page load
document.addEventListener('FinishInline', async function () {
    await displaySearchResults();
});

function makeUpvoteButton(button, map_id) {
    button.classList.remove('btn-success');
    button.classList.add('btn-outline-light');

    button.onclick = async function() {
        let count = button.querySelector('.upvote-count')
        count.textContent = parseInt(count.textContent, 10)+1;

        button.disabled = true;
        await runLoggedIn(async function (idToken) {
            const response = await fetch('/api/upvote', {
                method: 'POST',
                body: JSON.stringify({
                    'firebaseToken': idToken,
                    'mapId': map_id
                })
            });
            const result = await response.text();

            if (response.ok) {
                makeUnvoteButton(button, map_id);
            } else {
                console.error('Error upvoting: ', response)
                showError(result || 'An error occurred when upvoting.');
            }
        });
    };
    button.disabled = false;
}

function makeUnvoteButton(button, map_id) {
    button.classList.remove('btn-outline-light');
    button.classList.add('btn-success');

    button.onclick = async () => {
        let count = button.querySelector('.upvote-count')
        count.textContent = parseInt(count.textContent, 10)-1;

        button.disabled = true;
        await runLoggedIn(async function (idToken) {
            const response = await fetch('/api/unvote', {
                method: 'POST',
                body: JSON.stringify({
                    'firebaseToken': idToken,
                    'mapId': map_id
                })
            });
            const result = await response.text();

            if (response.ok) {
                makeUpvoteButton(button, map_id);
            } else {
                console.error('Error unvoting: ', response)
                showError(result || 'An error occurred when removing your upvote.');
            }
        });
    }
    button.disabled = false;
}

function makeDownloadButton(button, map_id) {
    button.textContent = 'Oneclick';
    button.classList.remove('btn-danger');
    button.classList.add('btn-primary');
    button.onclick = async () => {
        button.disabled = true;
        await runLoggedIn(async function (idToken) {
            const response = await fetch('/api/download', {
                method: 'POST',
                body: JSON.stringify({
                    'firebaseToken': idToken,
                    'mapId': map_id
                })
            });
            const result = await response.text();

            if (response.ok) {
                makeRemoveButton(button, map_id);
            } else {
                console.error('Error syncing downloading: ', response)
                showError(result || 'An error occurred when syncing downloading.');
            }
        });
        await downloadMap(button, map_id);
    }
    button.disabled = false;
}

function makeRemoveButton(button, map_id) {
    button.textContent = 'Remove';
    button.classList.remove('btn-primary');
    button.classList.add('btn-danger');

    button.onclick = async function() {
        button.disabled = true;
        await runLoggedIn(async function (idToken) {
            const response = await fetch('/api/remove', {
                method: 'POST',
                body: JSON.stringify({
                    'firebaseToken': idToken,
                    'mapId': map_id
                })
            });
            const result = await response.text();

            if (response.ok) {
                makeDownloadButton(button, map_id);
            } else {
                console.error('Error syncing removing: ', response)
                showError(result || 'An error occurred when syncing removing.');
            }
        });
        await removeMap(button, map_id)
    }
    button.disabled = false;
}