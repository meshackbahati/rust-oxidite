// Glitch effect on page load
document.addEventListener('DOMContentLoaded', function () {
    const form = document.getElementById('waitlistForm');
    const emailInput = document.getElementById('email');
    const messageDiv = document.getElementById('message');

    // Add glitch effect to hero title
    const heroTitle = document.querySelector('.hero h1');
    if (heroTitle) {
        setInterval(() => {
            if (Math.random() > 0.95) {
                heroTitle.style.animation = 'none';
                setTimeout(() => {
                    heroTitle.style.animation = 'glitch 3s infinite';
                }, 50);
            }
        }, 3000);
    }

    if (form) {
        form.addEventListener('submit', async function (e) {
            e.preventDefault();

            const email = emailInput.value.trim();

            if (!isValidEmail(email)) {
                showMessage('INVALID INPUT FORMAT', 'error');
                return;
            }

            // Add loading state
            const submitBtn = form.querySelector('button');
            const originalText = submitBtn.textContent;
            submitBtn.textContent = 'PROCESSING...';
            submitBtn.disabled = true;

            try {
                const response = await fetch('/api/waitlist', {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                    },
                    body: JSON.stringify({ email: email })
                });

                const data = await response.json();

                if (data.success) {
                    showMessage('ACCESS GRANTED // CONFIRMATION PENDING', 'success');
                    form.reset();
                    // Trigger success animation
                    triggerSuccessEffect();
                } else {
                    showMessage(data.message || 'SYSTEM ERROR // RETRY PROTOCOL INITIATED', 'error');
                }
            } catch (error) {
                console.error('Error:', error);
                showMessage('CONNECTION LOST // VERIFY NETWORK STATUS', 'error');
            } finally {
                submitBtn.textContent = originalText;
                submitBtn.disabled = false;
            }
        });
    }

    function isValidEmail(email) {
        const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
        return emailRegex.test(email);
    }

    function showMessage(text, type) {
        messageDiv.textContent = '';
        messageDiv.className = type;

        // Terminal-style typing effect
        let i = 0;
        const typeInterval = setInterval(() => {
            if (i < text.length) {
                messageDiv.textContent += text.charAt(i);
                i++;
            } else {
                clearInterval(typeInterval);
            }
        }, 30);

        // Hide message after 8 seconds
        setTimeout(() => {
            messageDiv.style.opacity = '0';
            setTimeout(() => {
                messageDiv.style.display = 'none';
                messageDiv.style.opacity = '1';
            }, 300);
        }, 8000);
    }

    function triggerSuccessEffect() {
        // Flash effect on success
        const hero = document.querySelector('.hero');
        hero.style.transition = 'all 0.1s';
        hero.style.boxShadow = '0 0 50px rgba(0, 255, 0, 0.8)';
        setTimeout(() => {
            hero.style.boxShadow = '';
        }, 200);
    }
});

// Random scan line glitches
setInterval(() => {
    if (Math.random() > 0.9) {
        const scanline = document.querySelector('body::before');
        if (scanline) {
            document.body.style.setProperty('--glitch-opacity', Math.random() * 0.3);
        }
    }
}, 2000);

// Smooth scrolling for anchor links
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
        e.preventDefault();
        const target = document.querySelector(this.getAttribute('href'));
        if (target) {
            target.scrollIntoView({
                behavior: 'smooth'
            });
        }
    });
});