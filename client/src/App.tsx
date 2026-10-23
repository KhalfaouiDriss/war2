import { useEffect, useState } from 'react';

function App() {
  const [wasm, setWasm] = useState<any>(null);

  useEffect(() => {
    // Load WASM module
    import('../pkg/war2_sim').then(module => {
      setWasm(module);
      console.log('WASM loaded:', module);
    }).catch(err => {
      console.error('Failed to load WASM:', err);
    });
  }, []);

  return (
    <div style={{ padding: '20px', fontFamily: 'system-ui, sans-serif' }}>
      <h1>War2 - Persistent World RTS</h1>
      <div style={{ marginTop: '20px', padding: '10px', background: '#2a2a4a', borderRadius: '4px' }}>
        <h2>Status</h2>
        <p>WASM: {wasm ? 'Loaded ✓' : 'Loading...'}</p>
        <p>Gateway: <span id="gateway-status">Connecting...</span></p>
      </div>
      <div style={{ marginTop: '20px' }}>
        <button onClick={() => console.log('Test')}>Test Button</button>
      </div>
    </div>
  );
}

export default App;