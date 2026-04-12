'use strict'

async function callApi(path, customOptions = {}) {
  const defaultOptions = {
    credentials: "include",
    headers: {
      "Content-Type": "application/json",
    }
  };
  const finalOptions = {
    ...defaultOptions,
    ...customOptions,
    headers: {
      ...defaultOptions.headers,
      ...customOptions.headers
    }
  };
  const res = await fetch(`/api/${path}`, finalOptions);

  let data = null;
  const text = await res.text();
  if (text) {
    try { data = JSON.parse(text); } catch (_) { }
  }

  return { ok: res.ok, status: res.status, data };
}