// Function to handle upvoting
function handleUpvote(event) {
    const button = event.currentTarget;
    const countSpan = button.querySelector('.upvote-count');
    let count = parseInt(countSpan.textContent, 10);
    count += 1;
    countSpan.textContent = count;

    // Optional: Change button appearance after upvote
    button.classList.remove('btn-outline-light');
    button.classList.add('btn-success');
    button.innerHTML = `<span class="bi bi-hand-thumbs-up-fill"></span> ${count}`;

    // Disable the button to prevent multiple upvotes
    button.disabled = true;
}

// Function to display search results
async function displaySearchResults(params) {
    const resultsContainer = document.getElementById('search-results');
    const noResultsContainer = document.getElementById('no-results');
    const template = document.getElementById('search-result-template');

    try {
        // Show a loading indicator (optional)
        resultsContainer.innerHTML = '<p>Loading results...</p>';
        noResultsContainer.style.display = 'none';

        // Fetch data from the /api/search endpoint
        let response;
        try {
             response = await fetch(`/api/search${params}`, {
                method: 'GET',
                headers: {
                    'Accept': 'application/json'
                }
            });
        } catch (e) {
            console.log(e);
        }

        if (response == null) {
            resultsContainer.innerHTML = '';
            document.getElementById('search-error').classList.remove('invisible');
            document.getElementById('search-error-text').textContent = "Failed to search, see console log";
            return
        }

        if (response.status === 429) {
            resultsContainer.innerHTML = '';
            document.getElementById('search-error').classList.remove('invisible');
            document.getElementById('search-error-text').innerHTML = 'Please stop spamming search requests!';
            return
        }

        if (!response.ok) {
            resultsContainer.innerHTML = '';
            document.getElementById('search-error').classList.remove('invisible');
            document.getElementById('search-error-text').textContent = 'Failed to search, see console log';
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

                // Populate the clone with actual data
                clone.querySelector('.card-title').textContent = map.song;
                clone.querySelectorAll('.card-text')[0].innerHTML = map.artist;
                clone.querySelectorAll('.card-text')[1].innerHTML = map.charter;
                clone.querySelectorAll('.card-text')[2].innerHTML = map.difficulties.map(d => d.display).join(", ");
                if (map.image != null) {
                    clone.querySelector('img').src = 'output/' + map.image;
                } else {
                    clone.querySelector('img').src = 'beatblocks.jpg';
                }
                clone.querySelector('a.btn').href = 'output/' + map.download;
                const button = clone.querySelector('.upvote-button');
                button.querySelector('.upvote-count').textContent = map.upvotes;
                button.addEventListener('click', handleUpvote);

                // Append the clone to the results container
                resultsContainer.appendChild(clone);
            });
        } else {
            // No results found
            noResultsContainer.classList.remove('invisible');
            resultsContainer.innerHTML = '';
        }
    } catch (error) {
        console.error('Error fetching search results:', error);
        resultsContainer.innerHTML = '';
        document.getElementById('search-error').classList.remove('invisible');
        document.getElementById('search-error-text').textContent = 'An error occurred while fetching search results. Please try again later.';
    }
}

// Initialize search results on page load
document.addEventListener('DOMContentLoaded', function() {
    displaySearchResults(window.location.search);
});