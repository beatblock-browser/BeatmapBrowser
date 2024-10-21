/**
 * Automatically replaces the header id with the header navbar,
 * and the footer id with the footer.
 * Prevents me having to write the same code on a bunch of pages.
 */
$(document).ready(function () {
    load_if_real('#navbar', 'templates/navbar.html');
    load_if_real('#searchbar', 'templates/searchbar.html');
    load_if_real('#search-result-template', 'templates/search_result.html');
    load_if_real('#search-result-template-long', 'templates/search_result.html', function () {
        $(this).children()[0].classList.remove('col-md-6');
    });
});

function load_if_real(name, file, callback = function () {}) {
    const element = $(name);
    if (element) {
        element.load(file, callback);
    }
}