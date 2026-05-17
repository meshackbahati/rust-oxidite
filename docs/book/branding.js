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
    label.textContent = "OXIDITE";

    brandLink.appendChild(logo);
    brandLink.appendChild(label);
    leftButtons.insertBefore(brandLink, leftButtons.firstChild);
  }

  function updateSEO() {
    const title = document.querySelector("h1")?.textContent || "Documentation";
    document.title = `${title} | Oxidite - Professional Rust Web Framework`;

    // Add meta description if not exists
    if (!document.querySelector('meta[name="description"]')) {
      const meta = document.createElement('meta');
      meta.name = "description";
      meta.content = "Official documentation for Oxidite, the high-performance, enterprise-ready web framework for Rust. Explore routing, ORM, real-time, and plugin systems.";
      document.head.appendChild(meta);
    }

    // Add OpenGraph tags
    const ogTags = [
      { property: 'og:title', content: document.title },
      { property: 'og:description', content: "Oxidite: Modern, Fast & Powerful Rust Web Framework." },
      { property: 'og:type', content: 'website' },
      { property: 'og:image', content: `${window.location.origin}${root}assets/oxidite-og.png` }
    ];

    ogTags.forEach(tag => {
      if (!document.querySelector(`meta[property="${tag.property}"]`)) {
        const meta = document.createElement('meta');
        meta.setAttribute('property', tag.property);
        meta.content = tag.content;
        document.head.appendChild(meta);
      }
    });
  }

  function updateLabels() {
    const title = document.querySelector("#mdbook-menu-bar .menu-title");
    if (title) {
      title.textContent = "Oxidite Handbook";
    }

    const search = document.getElementById("mdbook-searchbar");
    if (search) {
      search.setAttribute("placeholder", "Search the framework ...");
    }
  }

  document.addEventListener("DOMContentLoaded", () => {
    addBrand();
    updateSEO();
    updateLabels();
  });
})();
