use oxidite_template::{TemplateEngine, Context};

#[test]
fn test_inheritance() {
    let mut engine = TemplateEngine::new();

    // Base template
    engine.add_template("base.html", r#"
    <html>
    <head><title>{% block title %}Default Title{% endblock %}</title></head>
    <body>
        <div id="content">
            {% block content %}Default Content{% endblock %}
        </div>
        <footer>{% block footer %}Default Footer{% endblock %}</footer>
    </body>
    </html>
    "#).unwrap();

    // Child template
    engine.add_template("child.html", r#"
    {% extends "base.html" %}

    {% block title %}Child Title{% endblock %}

    {% block content %}
        <h1>Hello World</h1>
        <p>This is child content.</p>
    {% endblock %}
    "#).unwrap();

    let context = Context::new();
    let output = engine.render("child.html", &context).unwrap();

    assert!(output.contains("<title>Child Title</title>"));
    assert!(output.contains("<h1>Hello World</h1>"));
    assert!(output.contains("Default Footer"));
    assert!(!output.contains("Default Title"));
    assert!(!output.contains("Default Content"));
}

#[test]
fn test_include() {
    let mut engine = TemplateEngine::new();

    engine.add_template("header.html", "<h1>Header</h1>").unwrap();
    engine.add_template("page.html", r#"
    {% include "header.html" %}
    <p>Page Content</p>
    "#).unwrap();

    let context = Context::new();
    let output = engine.render("page.html", &context).unwrap();

    assert!(output.contains("<h1>Header</h1>"));
    assert!(output.contains("<p>Page Content</p>"));
}

#[test]
fn test_nested_inheritance() {
    let mut engine = TemplateEngine::new();

    engine.add_template("base.html", "Base: {% block content %}Base{% endblock %}").unwrap();
    engine.add_template("middle.html", "{% extends \"base.html\" %} {% block content %}Middle > {% block inner %}Inner{% endblock %}{% endblock %}").unwrap();
    engine.add_template("child.html", "{% extends \"middle.html\" %} {% block inner %}Child{% endblock %}").unwrap();

    let context = Context::new();
    // Note: Nested blocks in overrides are tricky. 
    // My implementation flattens blocks.
    // When rendering "child.html":
    // 1. Collect "inner" -> "Child"
    // 2. Render "middle.html"
    // 3. Middle extends Base.
    // 4. Collect "content" -> "Middle > {% block inner %}Inner{% endblock %}"
    // 5. Render "base.html"
    // 6. Base has block "content".
    // 7. Override found (from Middle). Render "Middle > {% block inner %}Inner{% endblock %}"
    // 8. "Middle > " rendered.
    // 9. Block "inner" found.
    // 10. Override found (from Child). Render "Child".
    // Result: "Base: Middle > Child"
    
    let output = engine.render("child.html", &context).unwrap();
    assert_eq!(output.trim(), "Base: Middle > Child");
}
