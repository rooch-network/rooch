import cbor from 'cbor'
import { RoochDataSource } from './rooch_datasource';
import { Wallet } from '../../wallet';
import { 
  RoochTransport, 
  PaginatedUTXOStateViews, 
  PaginatedInscriptionStateViews,
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
        // 注意：meta 字段应该不存在，因为解码失败
      });
  
      // 验证控制台警告信息
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
              metadata: "", // 注意这里的 metadata 为 null
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
        // 注意：meta 字段应该不存在，因为没有元数据
      });
    });
  });

});