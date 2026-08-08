/**
 * Servidor HTTP para el ecosistema civer.cloud
 * Routing virtual por hostname (Host header):
 *   ia.civer.cloud           -> ia.html  (Hub IA y landing de Civer)
 *   chat.civer.cloud         -> PROXY a 127.0.0.1:3002 (Antigravity Link Extension)
 *   manager.civer.cloud      -> index.html (sitio de descarga Civer Cloud Manager IDE)
 *
 * Auto-inicio configurado via Task Scheduler de Windows.
 * Ruta: C:\ProyectoCiverCloudUnificado\mesh-shared-vault\sitio-descarga\server.js
 */

const http = require('http');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

const PORT = 3000;
const SERVE_DIR = __dirname;
const ZIP_FILE = 'Civer-Cloud-Manager-IDE-v5.2.0-Oficial.zip';
const APP_VERSION = '5.2.0';
const CHAT_PROXY_TARGET = { host: '127.0.0.1', port: 3002 };

const MIME_TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.css':  'text/css',
  '.js':   'application/javascript',
  '.json': 'application/json',
  '.png':  'image/png',
  '.jpg':  'image/jpeg',
  '.ico':  'image/x-icon',
  '.exe':  'application/octet-stream',
  '.msi':  'application/octet-stream',
  '.apk':  'application/vnd.android.package-archive',
  '.zip':  'application/zip',
  '.svg':  'image/svg+xml',
};

function serveHtml(res, filePath) {
  fs.readFile(filePath, (err, data) => {
    if (err) { res.writeHead(404); res.end('Not found'); return; }
    res.writeHead(200, { 
      'Content-Type': 'text/html; charset=utf-8', 
      'Cache-Control': 'no-store, no-cache, must-revalidate, proxy-revalidate, max-age=0, s-maxage=0',
      'CDN-Cache-Control': 'no-store',
      'Cloudflare-CDN-Cache-Control': 'no-store',
      'Pragma': 'no-cache',
      'Expires': '0',
    });
    res.end(data);
  });
}

function proxyRequest(req, res, target) {
  const options = {
    hostname: target.host,
    port: target.port,
    path: req.url,
    method: req.method,
    headers: req.headers
  };
  
  const proxyReq = http.request(options, (proxyRes) => {
    res.writeHead(proxyRes.statusCode, proxyRes.headers);
    proxyRes.pipe(res, { end: true });
  });

  proxyReq.on('error', (e) => {
    res.writeHead(502);
    res.end(`Bad Gateway: Cannot connect to chat panel on port ${target.port}. Is the Antigravity Link extension running?`);
  });

  req.pipe(proxyReq, { end: true });
}

const server = http.createServer((req, res) => {
  const host = (req.headers.host || '').split(':')[0].toLowerCase();
  
  // chat.civer.cloud -> proxy a port 3002
  if (host.startsWith('chat.')) {
    return proxyRequest(req, res, CHAT_PROXY_TARGET);
  }
  if (host.startsWith('hypervisor.')) {
    return proxyRequest(req, res, { host: '127.0.0.1', port: 3005 });
  }
  if (host.startsWith('status.')) {
    return proxyRequest(req, res, { host: '127.0.0.1', port: 3004 });
  }
  if (host.startsWith('nodriza.')) {
    return proxyRequest(req, res, { host: '127.0.0.1', port: 5173 });
  }
  if (host.startsWith('omni-drive.')) {
    return proxyRequest(req, res, { host: '127.0.0.1', port: 3006 });
  }
  if (host.startsWith('qr.')) {
    return proxyRequest(req, res, { host: '127.0.0.1', port: 3007 });
  }
  if (host.startsWith('gptlab.')) {
    return proxyRequest(req, res, { host: '127.0.0.1', port: 3008 });
  }
  if (host.startsWith('profeonline.')) {
    return proxyRequest(req, res, { host: '127.0.0.1', port: 3009 });
  }
  if (host.startsWith('vps.')) {
    return proxyRequest(req, res, { host: '127.0.0.1', port: 3010 });
  }
  if (host.startsWith('streamit.')) {
    return proxyRequest(req, res, { host: '127.0.0.1', port: 8000 });
  }
  if (host.startsWith('md5.')) {
    return proxyRequest(req, res, { host: '127.0.0.1', port: 3011 });
  }
  if (host.startsWith('vault.')) {
    return proxyRequest(req, res, { host: '127.0.0.1', port: 5000 });
  }

  const isRoot = req.url === '/' || req.url === '' || req.url === '/index.html';

  // /hash endpoint - MD5 del instalador activo
  if (req.url === '/hash' || req.url === '/hash.json') {
    const zipPath = path.join(SERVE_DIR, ZIP_FILE);
    fs.stat(zipPath, (err, stat) => {
      if (err) {
        res.writeHead(404, { 'Content-Type': 'application/json' });
        return res.end(JSON.stringify({ error: 'ZIP no encontrado' }));
      }
      const hash = crypto.createHash('md5');
      const stream = fs.createReadStream(zipPath);
      stream.on('data', chunk => hash.update(chunk));
      stream.on('end', () => {
        const md5 = hash.digest('hex');
        res.writeHead(200, { 'Content-Type': 'application/json', 'Access-Control-Allow-Origin': '*' });
        res.end(JSON.stringify({
          version: APP_VERSION,
          file: ZIP_FILE,
          md5: md5,
          size_bytes: stat.size,
          size_mb: (stat.size / 1024 / 1024).toFixed(2) + ' MB',
          generated: stat.mtime
        }));
      });
      stream.on('error', () => {
        res.writeHead(500); res.end('Hash error');
      });
    });
    return;
  }

  // ia.civer.cloud -> ia.html
  if (isRoot && host.startsWith('ia.')) {
    return serveHtml(res, path.join(SERVE_DIR, 'ia.html'));
  }

  // Resto: estatico normal
  let urlPath = req.url.split('?')[0];
  if (urlPath === '/' || urlPath === '') { urlPath = '/index.html'; }

  const filePath = path.join(SERVE_DIR, urlPath);
  if (!filePath.startsWith(SERVE_DIR)) { res.writeHead(403); res.end('Forbidden'); return; }

  fs.stat(filePath, (err, stat) => {
    if (err || !stat.isFile()) {
      return serveHtml(res, path.join(SERVE_DIR, 'index.html'));
    }
    const ext = path.extname(filePath).toLowerCase();
    const contentType = MIME_TYPES[ext] || 'application/octet-stream';
    const readStream = fs.createReadStream(filePath);
    res.writeHead(200, {
      'Content-Type': contentType,
      'Content-Length': stat.size,
      'Access-Control-Allow-Origin': '*',
      'Cache-Control': 'no-store, no-cache, must-revalidate, proxy-revalidate, max-age=0, s-maxage=0',
      'CDN-Cache-Control': 'no-store',
      'Cloudflare-CDN-Cache-Control': 'no-store',
      'Pragma': 'no-cache',
      'Expires': '0'
    });
    readStream.pipe(res);
  });
});

// Soporte para WebSockets (necesario para el chat panel / CDP)
server.on('upgrade', (req, socket, head) => {
  const host = (req.headers.host || '').split(':')[0].toLowerCase();
  
  let target = null;
  if (host.startsWith('chat.')) {
    target = CHAT_PROXY_TARGET;
  } else if (host.startsWith('hypervisor.')) {
    target = { host: '127.0.0.1', port: 3005 };
  } else if (host.startsWith('status.')) {
    target = { host: '127.0.0.1', port: 3004 };
  } else if (host.startsWith('nodriza.')) {
    target = { host: '127.0.0.1', port: 5173 };
  } else if (host.startsWith('omni-drive.')) {
    target = { host: '127.0.0.1', port: 3006 };
  } else if (host.startsWith('qr.')) {
    target = { host: '127.0.0.1', port: 3007 };
  } else if (host.startsWith('gptlab.')) {
    target = { host: '127.0.0.1', port: 3008 };
  } else if (host.startsWith('profeonline.')) {
    target = { host: '127.0.0.1', port: 3009 };
  } else if (host.startsWith('vps.')) {
    target = { host: '127.0.0.1', port: 3010 };
  } else if (host.startsWith('streamit.')) {
    target = { host: '127.0.0.1', port: 8000 };
  } else if (host.startsWith('md5.')) {
    target = { host: '127.0.0.1', port: 3011 };
  } else if (host.startsWith('vault.')) {
    target = { host: '127.0.0.1', port: 5000 };
  }

  if (target) {
    const options = {
      hostname: target.host,
      port: target.port,
      path: req.url,
      method: req.method,
      headers: req.headers
    };
    
    const proxyReq = http.request(options);
    proxyReq.on('upgrade', (proxyRes, proxySocket, proxyHead) => {
      let headers = `HTTP/${req.httpVersion} 101 Switching Protocols\r\n`;
      for (let i = 0; i < proxyRes.rawHeaders.length; i += 2) {
        headers += `${proxyRes.rawHeaders[i]}: ${proxyRes.rawHeaders[i+1]}\r\n`;
      }
      headers += '\r\n';
      
      socket.write(headers);
      if (proxyHead && proxyHead.length) socket.write(proxyHead);
      
      proxySocket.pipe(socket);
      socket.pipe(proxySocket);
    });
    
    proxyReq.on('error', () => {
      socket.write(`HTTP/${req.httpVersion} 502 Bad Gateway\r\n\r\n`);
      socket.end();
    });
    
    proxyReq.end();
  } else {
    socket.destroy();
  }
});

server.listen(PORT, '0.0.0.0', () => {
  console.log(`[civer.cloud] Servidor activo en http://0.0.0.0:${PORT}`);
  console.log(`[civer.cloud]   manager.civer.cloud -> index.html`);
  console.log(`[civer.cloud]   ia.civer.cloud          -> ia.html`);
  console.log(`[civer.cloud]   chat.civer.cloud        -> PROXY a 127.0.0.1:3002`);
  console.log(`[civer.cloud]   hypervisor.civer.cloud  -> PROXY a 127.0.0.1:3005`);
  console.log(`[civer.cloud]   status.civer.cloud      -> PROXY a 127.0.0.1:3004`);
  console.log(`[civer.cloud]   nodriza.civer.cloud     -> PROXY a 127.0.0.1:5173`);
  console.log(`[civer.cloud]   omni-drive.civer.cloud  -> PROXY a 127.0.0.1:3006`);
  console.log(`[civer.cloud]   qr.civer.cloud          -> PROXY a 127.0.0.1:3007`);
  console.log(`[civer.cloud]   gptlab.civer.cloud      -> PROXY a 127.0.0.1:3008`);
  console.log(`[civer.cloud]   profeonline.civer.cloud -> PROXY a 127.0.0.1:3009`);
  console.log(`[civer.cloud]   vps.civer.cloud         -> PROXY a 127.0.0.1:3010`);
  console.log(`[civer.cloud]   streamit.civer.cloud    -> PROXY a 127.0.0.1:8000`);
});

server.on('error', (err) => {
  console.error('[civer.cloud] Error:', err.message);
  if (err.code === 'EADDRINUSE') { process.exit(1); }
});