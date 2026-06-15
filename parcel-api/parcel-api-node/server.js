import cluster from 'node:cluster';
import { availableParallelism } from 'node:os';
import Fastify from 'fastify';
import { StubParcelService } from './service.js';

const PORT = 8080;
const dataDir = process.env.PARCEL_DATA_DIR || '/parcel-data';

// Node's JS runs on one event loop, so a single process is core-bound. We fork
// one worker per available core (WEB_CONCURRENCY overrides); the OS load-balances
// keep-alive connections across the workers, which share the listen socket.
if (cluster.isPrimary) {
  const workers = Number(process.env.WEB_CONCURRENCY) || availableParallelism();
  console.log(`primary ${process.pid} forking ${workers} workers`);
  for (let i = 0; i < workers; i++) {
    cluster.fork();
  }
  cluster.on('exit', (worker, code, signal) => {
    console.log(`worker ${worker.process.pid} exited (${signal || code}); respawning`);
    cluster.fork();
  });
} else {
  startWorker();
}

async function startWorker() {
  console.log(`worker ${process.pid} loading parcel data from ${dataDir}`);
  const svc = new StubParcelService(dataDir);
  console.log(`worker ${process.pid} loaded ${svc.count} parcels`);

  const app = Fastify();

  // listParcels() returns a pre-serialized JSON string; send it verbatim so the
  // custom wire format is preserved (Fastify does not re-encode string payloads).
  app.post('/parcel-api/v1/parcel', (request, reply) => {
    reply.type('application/json').send(svc.listParcels());
  });

  app.get('/parcel-api/check/status', (request, reply) => {
    reply.type('text/plain; charset=UTF-8').send('parcel-api is on air');
  });

  app.get('/parcel-api/check', (request, reply) => {
    reply.type('text/plain; charset=UTF-8').send(memorySummary());
  });

  app.setNotFoundHandler((request, reply) => {
    reply.code(404).type('application/json').send('{"message":"Not Found"}');
  });

  await app.listen({ port: PORT, host: '0.0.0.0' });
  console.log(`parcel-api-node (worker ${process.pid}) listening on :${PORT}`);

  for (const sig of ['SIGINT', 'SIGTERM']) {
    process.on(sig, () => app.close().then(() => process.exit(0)));
  }
}

function memorySummary() {
  const m = process.memoryUsage();
  const mb = (b) => Math.floor(b / (1024 * 1024));
  return (
    'parcel-api\n\nMemory:\n' +
    `total: ${mb(m.heapTotal)}mb, free: ${mb(m.heapTotal - m.heapUsed)}mb, active: ${mb(m.heapUsed)}mb` +
    '\n\nVersion:\ndev\n'
  );
}
