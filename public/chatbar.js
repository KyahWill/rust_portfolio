// Chatbar Component JavaScript
(function() {
    'use strict';

    // Initialize chatbar when DOM is ready
    function initChatbar() {
        const chatbar = document.getElementById('chatbar');
        if (!chatbar) return;

        const toggle = chatbar.querySelector('.chatbar-toggle');
        const closeBtn = chatbar.querySelector('.chatbar-close');
        const panel = chatbar.querySelector('.chatbar-panel');
        const form = chatbar.querySelector('#chatbar-form');
        const input = chatbar.querySelector('#chatbar-input');
        const messagesContainer = chatbar.querySelector('#chatbar-messages');

        // Toggle chatbar
        function toggleChatbar() {
            const isOpen = chatbar.classList.toggle('open');
            toggle?.setAttribute('aria-expanded', String(isOpen));
            
            // Focus input when opening
            if (isOpen) {
                setTimeout(() => input?.focus(), 100);
            }
        }

        // Close chatbar
        function closeChatbar() {
            chatbar.classList.remove('open');
            toggle?.setAttribute('aria-expanded', 'false');
        }

        // Add message to chat
        function addMessage(text, type = 'user') {
            if (!messagesContainer) return;

            const messageDiv = document.createElement('div');
            messageDiv.className = `chatbar-message chatbar-message-${type}`;
            
            const p = document.createElement('p');
            p.textContent = text;
            messageDiv.appendChild(p);
            
            messagesContainer.appendChild(messageDiv);
            
            // Scroll to bottom
            messagesContainer.scrollTop = messagesContainer.scrollHeight;
        }

        // Handle form submission
        function handleSubmit(e) {
            e.preventDefault();
            
            const message = input?.value.trim();
            if (!message) return;

            // Add user message
            addMessage(message, 'user');
            
            // Clear input
            if (input) input.value = '';

            // Simulate response (you can replace this with actual API call)

            console.log('Sending message to API: ', message);
            fetch('/api/chat', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({ message }),
            })
            .then(response => response.json())
            .then(data => {
                console.log('API response: ', data);
                addMessage(data.response, 'system');
            })
            .catch(error => {
                console.error('Error sending message to API: ', error);
            });
        }

        // Event listeners
        toggle?.addEventListener('click', toggleChatbar);
        closeBtn?.addEventListener('click', closeChatbar);
        form?.addEventListener('submit', handleSubmit);

        // Close on Escape key
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape' && chatbar.classList.contains('open')) {
                closeChatbar();
            }
        });

        // Close when clicking outside (optional)
        document.addEventListener('click', (e) => {
            if (chatbar.classList.contains('open') && 
                !panel.contains(e.target) && 
                !toggle.contains(e.target)) {
                // Uncomment the line below if you want to close on outside click
                // closeChatbar();
            }
        });
    }

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initChatbar);
    } else {
        initChatbar();
    }
})();

