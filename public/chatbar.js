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

        // Navigate to a specific section on the page
        function navigateToSection(sectionId) {
            const section = document.getElementById(sectionId);
            if (section) {
                // Smooth scroll to the section
                section.scrollIntoView({ 
                    behavior: 'smooth', 
                    block: 'start' 
                });
                
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

        // Add message to chat
        function addMessage(text, type = 'user') {
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

                
                // Handle navigation if needed
                if (data.navigation && data.navigation.needed && data.navigation.sectionId) {
                    navigateToSection(data.navigation.sectionId);
                }
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

