(() => {
  let root = "";
  if (typeof path_to_root !== "undefined") {
    root = path_to_root;
  } else if (typeof window.path_to_root === "string") {
    root = window.path_to_root;
  }

  function addBrand() {
    const leftButtons = document.querySelector("#mdbook-menu-bar .left-buttons");
    if (!leftButtons || leftButtons.querySelector(".oxidite-docs-brand")) {
      return;
    }

    const brandLink = document.createElement("a");
    brandLink.className = "oxidite-docs-brand";
    brandLink.href = `${root}index.html`;
    brandLink.setAttribute("aria-label", "Oxidite Documentation Home");

    const logo = document.createElement("img");
    logo.src = `${root}assets/oxidite.svg`;
    logo.alt = "Oxidite logo";
    logo.loading = "eager";

    const label = document.createElement("span");
    label.textContent = "Oxidite";

    brandLink.appendChild(logo);
    brandLink.appendChild(label);
    leftButtons.insertBefore(brandLink, leftButtons.firstChild);
  }

  function updateLabels() {
    const title = document.querySelector("#mdbook-menu-bar .menu-title");
    if (title) {
      title.textContent = "Oxidite Documentation";
    }

    const search = document.getElementById("mdbook-searchbar");
    if (search) {
      search.setAttribute("placeholder", "Search Oxidite documentation ...");
    }
  }

  document.addEventListener("DOMContentLoaded", () => {
    addBrand();
    updateLabels();
  });
})();
