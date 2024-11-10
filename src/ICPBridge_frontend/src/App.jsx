import { useState, useEffect } from 'react';
import { ethers } from 'ethers';
import { ICPBridge_backend } from 'declarations/ICPBridge_backend';
import './index.scss';

function App() {
  const [dataHistoryList, setDataHistoryList] = useState([]);
  const [walletAddress, setWalletAddress] = useState('');
  const [provider, setProvider] = useState(null);
  const [signer, setSigner] = useState(null);
  const [transactionHash, setTransactionHash] = useState('');
  const [fromNetwork, setFromNetwork] = useState('Sepolia');
  const toNetwork = fromNetwork === 'Sepolia' ? 'BaseSepolia' : 'Sepolia';

  const networks = {
    Sepolia: {
      chainId: "0xAA36A7",
      usdcContractAddress: "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238",
      rpcUrl: "https://rpc.sepolia.org"
    },
    BaseSepolia: {
      chainId: "0x14A34",
      usdcContractAddress: "0x036CbD53842c5426634e7929541eC2318f3dCF7e",
      rpcUrl: "https://sepolia.base.org"
    }
  };

  const recipientAddress = "0x7f7346b12Ec7d7aa8fAD8Bc5E0a914919368a139";
  const usdcAbi = [
    "function transfer(address to, uint amount) public returns (bool)"
  ];

  useEffect(() => {
    async function initializeProvider() {
      let tempProvider;
      let tempSigner = null;

      if (window.ethereum == null) {
        console.log("MetaMask not installed; using default read-only provider for selected network");
        tempProvider = new ethers.JsonRpcProvider(networks[fromNetwork].rpcUrl);
      } else {
        tempProvider = new ethers.BrowserProvider(window.ethereum);
        try {
          tempSigner = await tempProvider.getSigner();
        } catch (error) {
          console.error("Signer could not be obtained:", error);
        }
      }

      setProvider(tempProvider);
      setSigner(tempSigner);
    }

    initializeProvider();
  }, [fromNetwork]);

  async function connectWallet() {
    if (window.ethereum) {
      try {
        await provider.send("eth_requestAccounts", []);
        const address = await signer.getAddress();
        setWalletAddress(address);
      } catch (error) {
        console.error("Failed to connect wallet:", error);
      }
    } else {
      alert("MetaMask not detected! Please install MetaMask.");
    }
  }

  async function switchNetwork() {
    const selectedNetwork = networks[fromNetwork];

    if (window.ethereum) {
      try {
        const network = await provider.getNetwork();
        if (network.chainId !== parseInt(selectedNetwork.chainId, 16)) {
          await window.ethereum.request({
            method: "wallet_addEthereumChain",
            params: [{
              chainId: selectedNetwork.chainId,
              chainName: fromNetwork === "Sepolia" ? "Sepolia" : "Base Sepolia",
              rpcUrls: [selectedNetwork.rpcUrl],
              nativeCurrency: { name: "Ethereum", symbol: "ETH", decimals: 18 },
            }],
          });
        }
      } catch (error) {
        console.error("Failed to switch network:", error);
      }
    }
  }

  async function sendUSDC() {
    if (signer) {
      try {
        await switchNetwork();

        const usdcContract = new ethers.Contract(
          networks[fromNetwork].usdcContractAddress,
          usdcAbi,
          signer
        );
        const amount = ethers.parseUnits("0.123", 6);
        const tx = await usdcContract.transfer(recipientAddress, amount);
        const receipt = await tx.wait();
        setTransactionHash(receipt.transactionHash);
      } catch (error) {
        console.error("Failed to send USDC:", error);
      }
    } else {
      alert("Please connect to the wallet first.");
    }
  }

  useEffect(() => {
    fetchDataHistory();
  }, []);

  function fetchDataHistory() {
    ICPBridge_backend.data_history().then((data_history) => {
      setDataHistoryList(data_history);
    });
  }

  return (
    <div className="app-container">
      <img src="/logo2.svg" alt="DFINITY logo" className="large-logo" />
      <div className="app-layout">
        <div className="app">
          <button onClick={connectWallet} className="primary-button">Connect Wallet</button>
          {walletAddress && <p className="wallet-info">Connected Wallet: {walletAddress}</p>}

          <div className="network-section">
            <div className="network-select">
              <label htmlFor="from-network">From:</label>
              <select
                id="from-network"
                value={fromNetwork}
                onChange={(e) => setFromNetwork(e.target.value)}
              >
                <option value="Sepolia">Sepolia</option>
                <option value="BaseSepolia">Base Sepolia</option>
              </select>
            </div>
            <div className="network-to">
              <label>To:</label>
              <span className={`network-to-text ${toNetwork.toLowerCase()}`}>
                {toNetwork === 'Sepolia' ? 'Sepolia' : 'Base Sepolia'}
              </span>
            </div>
          </div>

          <button onClick={sendUSDC} className="secondary-button">Send 0.123 USDC</button>
          {transactionHash && <p className="transaction-info">Transaction Hash: {transactionHash}</p>}
        </div>

        <div className="data-history-section">
          <h2>Data History</h2>
          <div className="data-history-list">
            {dataHistoryList.length > 0 ? (
              <ul>
                {dataHistoryList.map((item, index) => (
                  <li key={index}>{item}</li>
                ))}
              </ul>
            ) : (
              <p>No Data Yet</p>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export default App;
