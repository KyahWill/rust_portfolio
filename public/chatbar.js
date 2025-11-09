// Chatbar Component JavaScript
(function() {
    'use strict';

    // Markdown processor for chatbot responses
    function processMarkdown(text) {
        if (!text) return '';

        // Split text into lines for better processing
        const lines = text.split('\n');
        const processedLines = [];
        let inCodeBlock = false;
        let codeBlockContent = [];
        let inList = false;
        let listItems = [];

        function escapeHtml(str) {
            return str
                .replace(/&/g, '&amp;')
                .replace(/</g, '&lt;')
                .replace(/>/g, '&gt;');
        }

        function processInlineMarkdown(line) {
            // Escape HTML first
            let html = escapeHtml(line);

            // Code blocks are handled separately, so skip inline code if we're in a code block
            if (!inCodeBlock) {
                // Inline code (`code`)
                html = html.replace(/`([^`]+)`/g, '<code>$1</code>');
                
                // Bold (**text** or __text__) - but not inside code
                html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
                html = html.replace(/__(.+?)__/g, '<strong>$1</strong>');
                
                // Italic (*text* or _text_) - but not inside code or bold
                // Process italic after bold to avoid conflicts
                // Since bold is processed first, we can safely match single asterisks/underscores
                // Use word boundaries to avoid matching inside words
                html = html.replace(/\*([^*\n]+?)\*/g, function(match, content) {
                    // Skip if this was part of bold (shouldn't happen after bold processing)
                    if (match.includes('**')) return match;
                    return '<em>' + content + '</em>';
                });
                html = html.replace(/_([^_\n]+?)_/g, function(match, content) {
                    // Skip if this was part of bold
                    if (match.includes('__')) return match;
                    return '<em>' + content + '</em>';
                });
                
                // Links [text](url)
                html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener noreferrer">$1</a>');
            }

            return html;
        }

        for (let i = 0; i < lines.length; i++) {
            const line = lines[i];
            const trimmedLine = line.trim();

            // Handle code blocks
            if (trimmedLine.startsWith('```')) {
                if (inCodeBlock) {
                    // End code block
                    processedLines.push('<pre><code>' + escapeHtml(codeBlockContent.join('\n')) + '</code></pre>');
                    codeBlockContent = [];
                    inCodeBlock = false;
                } else {
                    // Start code block
                    inCodeBlock = true;
                }
                continue;
            }

            if (inCodeBlock) {
                codeBlockContent.push(line);
                continue;
            }

            // Handle headings
            if (trimmedLine.startsWith('### ')) {
                processedLines.push('<h3>' + processInlineMarkdown(trimmedLine.substring(4)) + '</h3>');
                continue;
            }
            if (trimmedLine.startsWith('## ')) {
                processedLines.push('<h2>' + processInlineMarkdown(trimmedLine.substring(3)) + '</h2>');
                continue;
            }
            if (trimmedLine.startsWith('# ')) {
                processedLines.push('<h1>' + processInlineMarkdown(trimmedLine.substring(2)) + '</h1>');
                continue;
            }

            // Handle lists
            if (/^[\-\*] (.+)$/.test(trimmedLine) || /^\d+\. (.+)$/.test(trimmedLine)) {
                const listItem = trimmedLine.replace(/^[\-\*] (.+)$/, '$1').replace(/^\d+\. (.+)$/, '$1');
                listItems.push('<li>' + processInlineMarkdown(listItem) + '</li>');
                inList = true;
                continue;
            }

            // End list if we were in one
            if (inList && trimmedLine === '') {
                processedLines.push('<ul>' + listItems.join('') + '</ul>');
                listItems = [];
                inList = false;
                continue;
            }

            // Regular paragraph line
            if (trimmedLine === '') {
                if (inList) {
                    processedLines.push('<ul>' + listItems.join('') + '</ul>');
                    listItems = [];
                    inList = false;
                }
                processedLines.push('');
            } else {
                if (inList) {
                    processedLines.push('<ul>' + listItems.join('') + '</ul>');
                    listItems = [];
                    inList = false;
                }
                processedLines.push(processInlineMarkdown(trimmedLine));
            }
        }

        // Close any open list or code block
        if (inList) {
            processedLines.push('<ul>' + listItems.join('') + '</ul>');
        }
        if (inCodeBlock) {
            processedLines.push('<pre><code>' + escapeHtml(codeBlockContent.join('\n')) + '</code></pre>');
        }

        // Group consecutive non-empty lines into paragraphs
        let result = '';
        let currentParagraph = [];

        for (let i = 0; i < processedLines.length; i++) {
            const line = processedLines[i];
            
            if (line === '') {
                if (currentParagraph.length > 0) {
                    result += '<p>' + currentParagraph.join(' ') + '</p>';
                    currentParagraph = [];
                }
            } else if (line.startsWith('<') && (line.startsWith('<h') || line.startsWith('<ul') || line.startsWith('<pre') || line.startsWith('<ol'))) {
                // Block-level element
                if (currentParagraph.length > 0) {
                    result += '<p>' + currentParagraph.join(' ') + '</p>';
                    currentParagraph = [];
                }
                result += line;
            } else {
                currentParagraph.push(line);
            }
        }

        // Close any remaining paragraph
        if (currentParagraph.length > 0) {
            result += '<p>' + currentParagraph.join(' ') + '</p>';
        }

        return result || '<p></p>';
    }

    // Storage key for chat messages
    const STORAGE_KEY = 'portfolio_chat_messages';

    // Save messages to localStorage
    function saveMessages(messages) {
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(messages));
        } catch (error) {
            console.error('Error saving messages to localStorage:', error);
        }
    }

    // Load messages from localStorage
    function loadMessages() {
        try {
            const stored = localStorage.getItem(STORAGE_KEY);
            if (stored) {
                return JSON.parse(stored);
            }
        } catch (error) {
            console.error('Error loading messages from localStorage:', error);
        }
        return [];
    }

    // Clear messages from localStorage
    function clearStoredMessages() {
        try {
            localStorage.removeItem(STORAGE_KEY);
        } catch (error) {
            console.error('Error clearing messages from localStorage:', error);
        }
    }

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
        const header = chatbar.querySelector('.chatbar-header');

        // Check if we're on desktop (min-width: 768px)
        function isDesktop() {
            return window.matchMedia('(min-width: 768px)').matches;
        }

        // Auto-open chatbar on desktop on first load (optional - can be removed if you want it closed by default)
        function handleDesktopMode() {
            if (isDesktop()) {
                // Check if user has previously closed it (stored in localStorage)
                const chatbarState = localStorage.getItem('chatbar_open');
                if (chatbarState === null) {
                    // First time - open by default
                    chatbar.classList.add('open');
                    toggle?.setAttribute('aria-expanded', 'true');
                } else {
                    // Restore previous state
                    if (chatbarState === 'true') {
                        chatbar.classList.add('open');
                        toggle?.setAttribute('aria-expanded', 'true');
                    } else {
                        chatbar.classList.remove('open');
                        toggle?.setAttribute('aria-expanded', 'false');
                    }
                }
            }
        }

        // Handle window resize
        function handleResize() {
            // On resize, maintain current state
            // If switching from mobile to desktop, restore desktop state
            if (isDesktop()) {
                const chatbarState = localStorage.getItem('chatbar_open');
                if (chatbarState === 'true') {
                    chatbar.classList.add('open');
                    toggle?.setAttribute('aria-expanded', 'true');
                } else if (chatbarState === 'false') {
                    chatbar.classList.remove('open');
                    toggle?.setAttribute('aria-expanded', 'false');
                }
            }
        }

        // Initialize desktop mode
        handleDesktopMode();
        window.addEventListener('resize', handleResize);

        // Toggle chatbar
        function toggleChatbar() {
            const isOpen = chatbar.classList.toggle('open');
            toggle?.setAttribute('aria-expanded', String(isOpen));
            
            // Save state to localStorage on desktop
            if (isDesktop()) {
                localStorage.setItem('chatbar_open', String(isOpen));
            }
            
            // Focus input when opening
            if (isOpen) {
                setTimeout(() => input?.focus(), 100);
            }
        }

        // Close chatbar
        function closeChatbar() {
            chatbar.classList.remove('open');
            toggle?.setAttribute('aria-expanded', 'false');
            
            // Save state to localStorage on desktop
            if (isDesktop()) {
                localStorage.setItem('chatbar_open', 'false');
            }
        }

        // Delete all chat messages
        function deleteChat() {
            if (confirm('Are you sure you want to delete all chat messages? This action cannot be undone.')) {
                // Clear messages array
                messages = [];
                
                // Clear localStorage
                clearStoredMessages();
                
                // Clear messages container
                if (messagesContainer) {
                    messagesContainer.innerHTML = '';
                }
                
                // Reset to welcome message
                messages = [{
                    text: 'Hello! How can I help you today?',
                    type: 'system',
                    timestamp: Date.now()
                }];
                saveMessages(messages);
                
                // Render welcome message
                renderMessage('Hello! How can I help you today?', 'system', false);
            }
        }

        // Create delete button dynamically
        function createDeleteButton() {
            if (!header) return;
            
            // Check if delete button already exists
            if (header.querySelector('.chatbar-delete')) return;
            
            // Create actions container if it doesn't exist
            let actionsContainer = header.querySelector('.chatbar-header-actions');
            if (!actionsContainer) {
                actionsContainer = document.createElement('div');
                actionsContainer.className = 'chatbar-header-actions';
                
                // Move close button into actions container if it exists
                if (closeBtn && closeBtn.parentNode === header) {
                    header.removeChild(closeBtn);
                    actionsContainer.appendChild(closeBtn);
                }
                
                header.appendChild(actionsContainer);
            }
            
            // Create delete button
            const deleteBtn = document.createElement('button');
            deleteBtn.className = 'chatbar-delete';
            deleteBtn.setAttribute('aria-label', 'Delete chat');
            deleteBtn.innerHTML = `
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="3 6 5 6 21 6"></polyline>
                    <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                </svg>
            `;
            deleteBtn.addEventListener('click', deleteChat);
            actionsContainer.appendChild(deleteBtn);
        }

        // Navigate to a specific page and section
        function navigateToPageAndSection(page, sectionId) {
            const currentPath = window.location.pathname;
            let targetPath = '/';
            
            // Determine target path based on page
            if (page === 'blogs') {
                targetPath = '/blogs';
            } else if (page && page.startsWith('/blogs/')) {
                // Handle blog post navigation (e.g., /blogs/welcome-to-my-blog)
                targetPath = page;
            } else if (page === 'index' || !page) {
                targetPath = '/';
            }
            
            // If we need to navigate to a different page
            if (currentPath !== targetPath) {
                // Build URL with section hash if needed
                let targetUrl = targetPath;
                if (sectionId && !targetPath.startsWith('/blogs/')) {
                    // Only add section hash if not navigating to a blog post
                    targetUrl += `#${sectionId}`;
                }
                
                // Navigate to the new page
                window.location.href = targetUrl;
            } else {
                // We're already on the correct page, just navigate to section
                if (sectionId) {
                    navigateToSection(sectionId);
                }
            }
        }
        
        // Navigate to a specific section on the current page
        function navigateToSection(sectionId) {
            const section = document.getElementById(sectionId);
            if (section) {
                // Smooth scroll to the section
                section.scrollIntoView({ 
                    behavior: 'smooth', 
                    block: 'start' 
                });
                
                // Update URL hash without triggering scroll
                window.history.pushState(null, '', `#${sectionId}`);
                
                // Optional: Highlight the section briefly
                section.style.transition = 'background-color 0.3s ease';
                const originalBg = section.style.backgroundColor;
                section.style.backgroundColor = 'rgba(47, 52, 58, 0.1)';
                
                setTimeout(() => {
                    section.style.backgroundColor = originalBg || '';
                }, 2000);
            } else {
                console.warn(`Section with ID "${sectionId}" not found`);
            }
        }
        
        // Handle hash navigation on page load
        function handleHashNavigation() {
            const hash = window.location.hash;
            if (hash) {
                const sectionId = hash.substring(1); // Remove the #
                setTimeout(() => {
                    navigateToSection(sectionId);
                }, 100); // Small delay to ensure page is fully loaded
            }
        }
        
        // Call handleHashNavigation after initialization
        handleHashNavigation();

        // Array to store messages in memory
        let messages = [];

        // Load messages from localStorage on initialization
        function loadStoredMessages() {
            if (!messagesContainer) return;
            
            const storedMessages = loadMessages();
            if (storedMessages.length > 0) {
                // Clear the initial welcome message if we have stored messages
                messagesContainer.innerHTML = '';
                messages = storedMessages;
                
                // Render all stored messages
                storedMessages.forEach(msg => {
                    renderMessage(msg.text, msg.type, false); // false = don't save again
                });
                
                // Scroll to bottom
                messagesContainer.scrollTop = messagesContainer.scrollHeight;
            } else {
                // No stored messages, keep the initial welcome message
                messages = [{
                    text: 'Hello! How can I help you today?',
                    type: 'system',
                    timestamp: Date.now()
                }];
                saveMessages(messages);
            }
        }

        // Render a message in the UI
        function renderMessage(text, type = 'user', shouldSave = true) {
            if (!messagesContainer) return;

            const messageDiv = document.createElement('div');
            messageDiv.className = `chatbar-message chatbar-message-${type}`;
            
            // Process markdown for system messages, plain text for user messages
            if (type === 'system' || type === 'other') {
                const contentDiv = document.createElement('div');
                contentDiv.className = 'chatbar-message-content';
                contentDiv.innerHTML = processMarkdown(text);
                messageDiv.appendChild(contentDiv);
            } else {
                const p = document.createElement('p');
                p.textContent = text;
                messageDiv.appendChild(p);
            }
            
            messagesContainer.appendChild(messageDiv);
            
            // Scroll to bottom
            messagesContainer.scrollTop = messagesContainer.scrollHeight;

            // Save to localStorage if needed
            if (shouldSave) {
                messages.push({
                    text: text,
                    type: type,
                    timestamp: Date.now()
                });
                saveMessages(messages);
            }
        }

        // Add message to chat (wrapper for renderMessage with saving enabled)
        function addMessage(text, type = 'user') {
            renderMessage(text, type, true);
        }

        // Load stored messages on initialization
        loadStoredMessages();

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

                
                // Handle navigation if needed
                if (data.navigation && data.navigation.needed) {
                    const page = data.navigation.page || null;
                    const sectionId = data.navigation.sectionId || null;
                    navigateToPageAndSection(page, sectionId);
                }
            })
            .catch(error => {
                console.error('Error sending message to API: ', error);
            });
        }

        // Create delete button
        createDeleteButton();

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

