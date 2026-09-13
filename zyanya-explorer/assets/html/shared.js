// Zyanya Shared UI and Navigation Script
document.addEventListener('DOMContentLoaded', () => {
    // Accessible Mobile Navigation Toggle
    const navToggle = document.getElementById('nav-toggle');
    const siteNav = document.getElementById('site-nav');
    if (navToggle && siteNav) {
        navToggle.addEventListener('click', () => {
            const isOpen = siteNav.classList.toggle('open');
            navToggle.classList.toggle('open', isOpen);
            navToggle.setAttribute('aria-expanded', isOpen);
        });

        document.addEventListener('click', (e) => {
            if (!siteNav.contains(e.target) && !navToggle.contains(e.target) && siteNav.classList.contains('open')) {
                siteNav.classList.remove('open');
                navToggle.classList.remove('open');
                navToggle.setAttribute('aria-expanded', 'false');
            }
        });

        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && siteNav.classList.contains('open')) {
                siteNav.classList.remove('open');
                navToggle.classList.remove('open');
                navToggle.setAttribute('aria-expanded', 'false');
                navToggle.focus();
            }
        });
    }

    // Auto-inject Brand SVGs
    const logoContainers = document.querySelectorAll('.brand-logo, #logo-container, #explorer-logo');
    if (logoContainers.length > 0) {
        fetch('/brand/zyanya-logo.svg')
            .then(r => r.text())
            .then(svg => {
                logoContainers.forEach(el => { el.innerHTML = svg; });
            })
            .catch(() => {});
    }

    // Live Consensus Sync Pill Update
    fetch('/api/info')
        .then(r => r.json())
        .then(info => {
            if (!info) return;
            const netLabel = document.getElementById('header-network-name');
            if (netLabel && info.network) {
                netLabel.textContent = `${info.network.toUpperCase()} · 1 BPS`;
            }
        })
        .catch(() => {});
});
