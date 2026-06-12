interface SignalData {
    channel: string;
    message: string;
    signal: string;
    value: number;
    allowBackend: boolean;
}

type WsMessage =
    | { type: 'init'; signals: any }
    | { type: 'clientUpdate'; signal: string; value: number }
    | { type: 'setArbitration'; signal: string; allowBackend: boolean }
    | { type: 'stateChanged'; signal: string; value: number };

const statusEl = document.getElementById('status')!;
const viewControls = document.getElementById('view-controls')!;
const viewMonitor = document.getElementById('view-monitor')!;
const monitorLog = document.getElementById('monitor-log')!;
const navControls = document.getElementById('nav-controls')!;
const navMonitor = document.getElementById('nav-monitor')!;
const clearMonitorBtn = document.getElementById('clear-monitor')!;
const globalToggle = document.getElementById('global-toggle') as HTMLInputElement;
const globalToggleContainer = document.getElementById('global-toggle-container')!;

let socket: WebSocket | null = null;
const signals: Map<string, SignalData> = new Map();

// Navigation Logic
navControls.onclick = () => switchPage('controls');
navMonitor.onclick = () => switchPage('monitor');
clearMonitorBtn.onclick = () => {
    monitorLog.innerHTML = '<div class="text-gray-500 italic">Listening for updates...</div>';
};

function switchPage(page: 'controls' | 'monitor') {
    if (page === 'controls') {
        viewControls.classList.remove('hidden');
        viewMonitor.classList.add('hidden');
        navControls.classList.add('bg-blue-600', 'text-white');
        navControls.classList.remove('hover:bg-gray-700', 'text-gray-300');
        navMonitor.classList.remove('bg-blue-600', 'text-white');
        navMonitor.classList.add('hover:bg-gray-700', 'text-gray-300');
        globalToggleContainer.classList.remove('hidden');
    } else {
        viewControls.classList.add('hidden');
        viewMonitor.classList.remove('hidden');
        navMonitor.classList.add('bg-blue-600', 'text-white');
        navMonitor.classList.remove('hover:bg-gray-700', 'text-gray-300');
        navControls.classList.remove('bg-blue-600', 'text-white');
        navControls.classList.add('hover:bg-gray-700', 'text-gray-300');
        globalToggleContainer.classList.add('hidden');
    }
}

function connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const host = window.location.host; // This includes the port if present
    const wsUrl = `${protocol}//${host}/ws`;
    
    console.log(`Connecting to WebSocket at: ${wsUrl}`);
    socket = new WebSocket(wsUrl);

    socket.onopen = () => {
        console.log('WebSocket connected');
        statusEl.textContent = 'Connected';
        statusEl.className = 'text-green-500';
    };

    socket.onclose = (event) => {
        console.log(`WebSocket closed: ${event.code} ${event.reason}`);
        statusEl.textContent = 'Disconnected - Retrying...';
        statusEl.className = 'text-red-500';
        setTimeout(connect, 2000);
    };

    socket.onerror = (error) => {
        console.error('WebSocket error:', error);
    };

    socket.onmessage = (event) => {
        console.log('Received message:', event.data);
        try {
            const msg: WsMessage = JSON.parse(event.data);
            handleMessage(msg);
            if (msg.type === 'stateChanged') {
                addToMonitor(msg.signal, msg.value);
            }
        } catch (e) {
            console.error('Failed to parse message:', e, event.data);
        }
    };
}

function addToMonitor(signal: string, value: number) {
    if (monitorLog.children.length > 0 && monitorLog.children[0].classList.contains('italic')) {
        monitorLog.innerHTML = '';
    }

    const entry = document.createElement('div');
    entry.className = 'flex items-center space-x-2 border-b border-gray-800 py-1 hover:bg-white/5 transition-colors';
    const timestamp = new Date().toLocaleTimeString();
    
    entry.innerHTML = `
        <span class="text-gray-500 min-w-[100px]">[${timestamp}]</span>
        <span class="text-blue-400 font-bold min-w-[200px]">${signal}</span>
        <span class="text-green-400 font-mono">→</span>
        <span class="text-yellow-400 font-bold">${value}</span>
    `;

    monitorLog.prepend(entry);

    // Keep only last 100 entries
    if (monitorLog.children.length > 100) {
        monitorLog.removeChild(monitorLog.lastChild!);
    }
}

globalToggle.onchange = () => {
    const isChecked = globalToggle.checked;
    console.log(`Setting global backend control: ${isChecked}`);
    
    signals.forEach((_, id) => {
        const toggle = document.getElementById(`toggle-${id}`) as HTMLInputElement;
        if (toggle) {
            if (toggle.checked !== isChecked) {
                toggle.checked = isChecked;
                sendArbitration(id, isChecked);
            }
        }
    });
};

function handleMessage(msg: WsMessage) {
    console.log('Handling message type:', msg.type);
    switch (msg.type) {
        case 'init':
            renderDashboard(msg.signals);
            break;
        case 'stateChanged':
            updateSignalValue(msg.signal, msg.value);
            break;
    }
}

function renderDashboard(data: any) {
    viewControls.innerHTML = '';
    signals.clear();

    for (const channel in data) {
        for (const message in data[channel]) {
            for (const signal in data[channel][message]) {
                const value = data[channel][message][signal];
                const id = signal; // Assuming signal labels are unique enough or we use full path
                signals.set(id, { channel, message, signal, value, allowBackend: true });
                
                const card = createSignalCard(id, channel, message, signal, value);
                viewControls.appendChild(card);
            }
        }
    }
}

function createSignalCard(id: string, channel: string, message: string, signal: string, value: number) {
    const card = document.createElement('div');
    card.className = 'bg-gray-800 p-4 rounded-lg shadow-lg border border-gray-700';
    card.id = `card-${id}`;

    card.innerHTML = `
        <div class="flex justify-between items-start mb-4">
            <div>
                <h3 class="font-bold text-lg">${signal}</h3>
                <p class="text-xs text-gray-400">${channel} / ${message}</p>
            </div>
            <div class="flex items-center space-x-2">
                <span class="text-xs text-gray-400">Backend Control</span>
                <input type="checkbox" checked id="toggle-${id}" class="w-4 h-4 text-blue-600 bg-gray-700 border-gray-600 rounded">
            </div>
        </div>
        <div class="flex items-center space-x-4">
            <div class="flex-1">
                <input type="number" id="input-${id}" value="${value}" class="w-full bg-gray-900 border border-gray-700 rounded px-3 py-2 text-xl font-mono">
            </div>
            <div class="flex flex-col space-y-1">
                <button id="up-${id}" class="bg-gray-700 hover:bg-gray-600 px-2 py-1 rounded">▲</button>
                <button id="down-${id}" class="bg-gray-700 hover:bg-gray-600 px-2 py-1 rounded">▼</button>
            </div>
        </div>
    `;

    const input = card.querySelector(`#input-${id}`) as HTMLInputElement;
    const toggle = card.querySelector(`#toggle-${id}`) as HTMLInputElement;
    const upBtn = card.querySelector(`#up-${id}`) as HTMLButtonElement;
    const downBtn = card.querySelector(`#down-${id}`) as HTMLButtonElement;

    input.onchange = () => sendUpdate(id, parseFloat(input.value));
    toggle.onchange = () => sendArbitration(id, toggle.checked);
    
    upBtn.onclick = () => {
        input.value = (parseFloat(input.value) + 1).toString();
        sendUpdate(id, parseFloat(input.value));
    };
    downBtn.onclick = () => {
        input.value = (parseFloat(input.value) - 1).toString();
        sendUpdate(id, parseFloat(input.value));
    };

    // Keyboard support
    input.onkeydown = (e) => {
        if (e.key === 'ArrowUp') {
            e.preventDefault();
            upBtn.click();
        } else if (e.key === 'ArrowDown') {
            e.preventDefault();
            downBtn.click();
        }
    };

    return card;
}

function updateSignalValue(id: string, value: number) {
    const signalData = signals.get(id);
    if (!signalData) return;

    signalData.value = value;
    const input = document.getElementById(`input-${id}`) as HTMLInputElement;
    if (input) {
        input.value = value.toString();
    }
}

function sendUpdate(signal: string, value: number) {
    if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify({ type: 'clientUpdate', signal, value }));
    }
}

function sendArbitration(signal: string, allowBackend: boolean) {
    const signalData = signals.get(signal);
    if (signalData) {
        signalData.allowBackend = allowBackend;
    }
    if (socket && socket.readyState === WebSocket.OPEN) {
        socket.send(JSON.stringify({ type: 'setArbitration', signal, allowBackend }));
    }
}

connect();
