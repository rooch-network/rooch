import cbor from 'cbor'
import { RoochDataSource } from './rooch_datasource';
import { Wallet } from '../../wallet';
import { 
  RoochTransport, 
  PaginatedUTXOStateViews, 
  PaginatedInscriptionStateViews,
  UTXOStateView,
} from '@roochnetwork/rooch-sdk';

class MockRoochTransport implements RoochTransport {
  private mockResponses: Map<string, any> = new Map();

  setMockResponse(method: string, response: any) {
    this.mockResponses.set(method, response);
  }

  async request<T>({ method }: { method: string; params: unknown[] }): Promise<T> {
    const response = this.mockResponses.get(method);
    if (response) {
      return response as T;
    }
    throw new Error(`No mock response set for method: ${method}`);
  }
}

describe('RoochDataSource', () => {
  let instance: RoochDataSource;
  let wallet: Wallet;
  let mockTransport: MockRoochTransport;

  beforeEach(() => {
    wallet = new Wallet({
      wif: 'cNGdjKojxE7nCcYdK34d12cdYTzBdDV4VdXdbpG7SHGTRWuCxpAW',
      network: "testnet",
      type: 'taproot',
    });

    mockTransport = new MockRoochTransport();
    instance = new RoochDataSource({ network: 'testnet', transport: mockTransport });
  });

  describe('getBalance', () => {
    it('should return the correct balance for a given address', async () => {
      if (!wallet.selectedAddress) {
        throw new Error('no selected address');
      }

      const mockResponse: PaginatedUTXOStateViews = {
        data: [
          {
            value: {
              value: '1000000',
              bitcoin_txid: 'mock_txid',
              seals: 'mock_seals',
              txid: 'mock_txid',
              vout: 0
            },
            created_at: 'mock_date',
            flag: 0,
            id: 'mock_id',
            object_type: 'mock_type',
            owner: wallet.selectedAddress,
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: 'mock_date'
          }
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockResponse);

      const balance = await instance.getBalance({ address: wallet.selectedAddress });
      expect(balance).toBe(1000000);
    });

    it('should return 0 balance for a new address', async () => {
      const wallet = new Wallet({
        bip39: 'right second until palace kid wear tennis phone bike broccoli oval saddle',
        network: "testnet",
        type: 'taproot',
      });

      const account = wallet.generateAddress('taproot', 2, 0);
      if (!account.address) {
        throw new Error('no selected address');
      }

      const mockResponse: PaginatedUTXOStateViews = {
        data: [],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryUTXOs', mockResponse);

      const balance = await instance.getBalance({ address: account.address });
      expect(balance).toBe(0);
    });
  });
  
  describe('getInscription', () => {
    it('should successfully get an inscription without decoding metadata', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const mockResponse: PaginatedInscriptionStateViews = {
        data: [
          {
            owner: 'mockOwner',
            value: {
              bitcoin_txid: 'mocktxid',
              index: 0,
              inscription_number: 1,
              body: 'bW9ja0JvZHk=', // base64 encoded 'mockBody'
              content_type: 'text/plain',
              txid: 'mocktxid',
              offset: '0',
              metadata: 'bW9ja01ldGFkYXRh', // base64 encoded 'mockMetadata'
              parents: '',
              is_curse: false,
              sequence_number: 0,
            },
            created_at: '2023-01-01T00:00:00Z',
            flag: 0,
            id: 'mock_id',
            object_type: 'mock_type',
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: '2023-01-01T00:00:00Z'
          },
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);

      const result = await instance.getInscription({ id: mockInscriptionId, decodeMetadata: false });

      expect(result).toEqual({
        id: 'mocktxidi0',
        number: 1,
        owner: 'mockOwner',
        mediaContent: 'bW9ja0JvZHk=',
        mediaSize: 8,
        mediaType: 'text/plain',
        timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
        genesis: 'mocktxid',
        outpoint: 'mocktxid:0',
        fee: 0,
        height: 0,
        sat: 0,
      });
    });

    it('should successfully get an inscription and decode metadata', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const mockMetadata = {
        name: "Test Inscription",
        description: "This is a test inscription",
        attributes: [
          { trait_type: "Type", value: "Test" },
          { trait_type: "Version", value: "1.0" }
        ]
      };
      
      // Encode the metadata using CBOR
      const encodedMetadata = cbor.encode(mockMetadata);
      const base64EncodedMetadata = Buffer.from(encodedMetadata).toString('base64');

      const mockResponse: PaginatedInscriptionStateViews = {
        data: [
          {
            owner: 'mockOwner',
            value: {
              bitcoin_txid: 'mocktxid',
              index: 0,
              inscription_number: 1,
              body: 'bW9ja0JvZHk=', // base64 encoded 'mockBody'
              content_type: 'text/plain',
              txid: 'mocktxid',
              offset: '0',
              metadata: base64EncodedMetadata,
              parents: '',
              is_curse: false,
              sequence_number: 0,
            },
            created_at: '2023-01-01T00:00:00Z',
            flag: 0,
            id: 'mock_id',
            object_type: 'mock_type',
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: '2023-01-01T00:00:00Z'
          },
        ],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);

      const result = await instance.getInscription({ id: mockInscriptionId, decodeMetadata: true });

      expect(result).toEqual({
        id: 'mocktxidi0',
        number: 1,
        owner: 'mockOwner',
        mediaContent: 'bW9ja0JvZHk=',
        mediaSize: 8,
        mediaType: 'text/plain',
        timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
        genesis: 'mocktxid',
        outpoint: 'mocktxid:0',
        fee: 0,
        height: 0,
        sat: 0,
        meta: mockMetadata,
      });
    });

    it('should throw an error when trying to get a non-existent inscription', async () => {
      const mockInscriptionId = 'nonexistenttxidi0';
      
      const mockResponse: PaginatedInscriptionStateViews = {
        data: [],
        has_next_page: false,
        next_cursor: null
      };

      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);

      await expect(instance.getInscription({ id: mockInscriptionId, decodeMetadata: false }))
        .rejects.toThrow(`Inscription with id ${mockInscriptionId} not found`);
    });

    it('should handle metadata decoding failure', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const invalidEncodedMetadata = 'invalidCBORdata';

      jest.spyOn(console, 'warn').mockImplementation(() => {});
  
      const mockResponse: PaginatedInscriptionStateViews = {
        data: [
          {
            owner: 'mockOwner',
            value: {
              bitcoin_txid: 'mocktxid',
              index: 0,
              inscription_number: 1,
              body: 'bW9ja0JvZHk=', // base64 encoded 'mockBody'
              content_type: 'text/plain',
              txid: 'mocktxid',
              offset: '0',
              metadata: invalidEncodedMetadata,
              parents: '',
              is_curse: false,
              sequence_number: 0,
            },
            created_at: '2023-01-01T00:00:00Z',
            flag: 0,
            id: 'mock_id',
            object_type: 'mock_type',
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: '2023-01-01T00:00:00Z'
          },
        ],
        has_next_page: false,
        next_cursor: null
      };
  
      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);
  
      const result = await instance.getInscription({ id: mockInscriptionId, decodeMetadata: true });
  
      expect(result).toEqual({
        id: 'mocktxidi0',
        number: 1,
        owner: 'mockOwner',
        mediaContent: 'bW9ja0JvZHk=',
        mediaSize: 8,
        mediaType: 'text/plain',
        timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
        genesis: 'mocktxid',
        outpoint: 'mocktxid:0',
        fee: 0,
        height: 0,
        sat: 0,
      });
  
      expect(console.warn).toHaveBeenCalledWith(expect.stringContaining('Failed to decode CBOR metadata for inscription mocktxidi0'));
    });
  
    it('should handle inscription without metadata', async () => {
      const mockInscriptionId = 'mocktxidi0';
  
      const mockResponse: PaginatedInscriptionStateViews = {
        data: [
          {
            owner: 'mockOwner',
            value: {
              bitcoin_txid: 'mocktxid',
              index: 0,
              inscription_number: 1,
              body: 'bW9ja0JvZHk=', // base64 encoded 'mockBody'
              content_type: 'text/plain',
              txid: 'mocktxid',
              offset: '0',
              metadata: "", 
              parents: '',
              is_curse: false,
              sequence_number: 0,
            },
            created_at: '2023-01-01T00:00:00Z',
            flag: 0,
            id: 'mock_id',
            object_type: 'mock_type',
            size: '1',
            state_index: '0',
            tx_order: '0',
            updated_at: '2023-01-01T00:00:00Z'
          },
        ],
        has_next_page: false,
        next_cursor: null
      };
  
      mockTransport.setMockResponse('btc_queryInscriptions', mockResponse);
  
      const result = await instance.getInscription({ id: mockInscriptionId, decodeMetadata: true });
  
      expect(result).toEqual({
        id: 'mocktxidi0',
        number: 1,
        owner: 'mockOwner',
        mediaContent: 'bW9ja0JvZHk=',
        mediaSize: 8,
        mediaType: 'text/plain',
        timestamp: new Date('2023-01-01T00:00:00Z').getTime(),
        genesis: 'mocktxid',
        outpoint: 'mocktxid:0',
        fee: 0,
        height: 0,
        sat: 0,
      });
    });
  });

  describe('getInscriptionUTXO', () => {
    it('should successfully get an inscription UTXO', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const mockUTXOState: UTXOStateView = {
        created_at: '2023-01-01T00:00:00Z',
        flag: 0,
        id: 'mock_id',
        object_type: 'mock_type',
        owner: 'mock_owner',
        owner_bitcoin_address: 'mock_bitcoin_address',
        size: '100',
        state_index: '0',
        tx_order: '0',
        updated_at: '2023-01-01T00:00:00Z',
        value: {
          bitcoin_txid: 'mocktxid',
          seals: 'mock_seals',
          txid: 'mocktxid',
          value: '1000',
          vout: 0
        }
      };

      mockTransport.setMockResponse('btc_queryInscriptions', {
        data: [{
          value: {
            bitcoin_txid: 'mocktxid',
            index: 0,
            outpoint: 'mocktxid:0'
          }
        }],
        has_next_page: false,
        next_cursor: null
      });

      mockTransport.setMockResponse('btc_queryUTXOs', {
        data: [mockUTXOState],
        has_next_page: false,
        next_cursor: null
      });

      const result = await instance.getInscriptionUTXO({ id: mockInscriptionId });

      expect(result).toEqual({
        n: 0,
        txid: 'mocktxid',
        sats: 1000,
        scriptPubKey: {
          asm: '',
          desc: '',
          hex: '',
          address: 'mock_bitcoin_address',
          type: 'p2tr',
        },
        safeToSpend: true,
        confirmation: -1,
      });
    });

    it('should throw an error when inscription does not exist', async () => {
      const mockInscriptionId = 'nonexistenttxidi0';
    
      // Mock empty response for inscription query
      mockTransport.setMockResponse('btc_queryInscriptions', {
        data: [],
        has_next_page: false,
        next_cursor: null
      });
    
      await expect(instance.getInscriptionUTXO({ id: mockInscriptionId }))
        .rejects.toThrow(`Inscription with id ${mockInscriptionId} not found`);
    });

    it('should throw an error when UTXO does not exist', async () => {
      const mockInscriptionId = 'mocktxidi0';
    
      mockTransport.setMockResponse('btc_queryInscriptions', {
        data: [{
          value: {
            bitcoin_txid: 'mocktxid',
            index: 0,
            outpoint: 'mocktxid:0'
          }
        }],
        has_next_page: false,
        next_cursor: null
      });
    
      // Mock empty response for UTXO query
      mockTransport.setMockResponse('btc_queryUTXOs', {
        data: [],
        has_next_page: false,
        next_cursor: null
      });
    
      await expect(instance.getInscriptionUTXO({ id: mockInscriptionId }))
        .rejects.toThrow(`UTXO for inscription ${mockInscriptionId} not found`);
    });

    it('should throw an error when UTXO value exceeds safe integer range', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const mockUTXOState: UTXOStateView = {
        created_at: '2023-01-01T00:00:00Z',
        flag: 0,
        id: 'mock_id',
        object_type: 'mock_type',
        owner: 'mock_owner',
        owner_bitcoin_address: 'mock_bitcoin_address',
        size: '100',
        state_index: '0',
        tx_order: '0',
        updated_at: '2023-01-01T00:00:00Z',
        value: {
          bitcoin_txid: 'mocktxid',
          seals: 'mock_seals',
          txid: 'mocktxid',
          value: '9007199254740992', // Exceeds Number.MAX_SAFE_INTEGER
          vout: 0
        }
      };
    
      mockTransport.setMockResponse('btc_queryInscriptions', {
        data: [{
          value: {
            bitcoin_txid: 'mocktxid',
            index: 0,
            outpoint: 'mocktxid:0'
          }
        }],
        has_next_page: false,
        next_cursor: null
      });
    
      mockTransport.setMockResponse('btc_queryUTXOs', {
        data: [mockUTXOState],
        has_next_page: false,
        next_cursor: null
      });
    
      await expect(instance.getInscriptionUTXO({ id: mockInscriptionId }))
        .rejects.toThrow('Failed to convert UTXO value to number: Error: UTXO value exceeds safe integer range');
    });

    it('should throw an error when UTXO value cannot be converted to a number', async () => {
      const mockInscriptionId = 'mocktxidi0';
      const mockUTXOState: UTXOStateView = {
        created_at: '2023-01-01T00:00:00Z',
        flag: 0,
        id: 'mock_id',
        object_type: 'mock_type',
        owner: 'mock_owner',
        owner_bitcoin_address: 'mock_bitcoin_address',
        size: '100',
        state_index: '0',
        tx_order: '0',
        updated_at: '2023-01-01T00:00:00Z',
        value: {
          bitcoin_txid: 'mocktxid',
          seals: 'mock_seals',
          txid: 'mocktxid',
          value: 'not-a-number', // Invalid value
          vout: 0
        }
      };
    
      mockTransport.setMockResponse('btc_queryInscriptions', {
        data: [{
          value: {
            bitcoin_txid: 'mocktxid',
            index: 0,
            outpoint: 'mocktxid:0'
          }
        }],
        has_next_page: false,
        next_cursor: null
      });
    
      mockTransport.setMockResponse('btc_queryUTXOs', {
        data: [mockUTXOState],
        has_next_page: false,
        next_cursor: null
      });
    
      await expect(instance.getInscriptionUTXO({ id: mockInscriptionId }))
        .rejects.toThrow('Failed to convert UTXO value to number: SyntaxError: Cannot convert not-a-number to a BigInt');
    });
  });

});