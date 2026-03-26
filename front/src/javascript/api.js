'use strict'

async function callApi(path, options) {
  const res = await fetch(`http://127.0.0.1:8080/api/${path}`, options);

  let data = null;
  const text = await res.text();
  if (text) {
    try { data = JSON.parse(text); } catch (_) {}
  }

  return { ok: res.ok, status: res.status, data };
}