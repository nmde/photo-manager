export function ab2b64(arrayBuffer: ArrayBuffer) {
  return btoa(String.fromCharCode.apply(null, new Uint8Array(arrayBuffer)));
}

export function b642ab(base64string: string) {
  return Uint8Array.from(atob(base64string), c => c.codePointAt(0) ?? 0);
}

export async function encrypt(
  text: string,
  key: CryptoKey,
  iv = crypto.getRandomValues(new Uint8Array(12)),
) {
  return {
    text: ab2b64(
      await crypto.subtle.encrypt(
        {
          name: 'AES-GCM',
          iv,
        },
        key,
        new TextEncoder().encode(text),
      ),
    ),
    iv,
  };
}

export async function decrypt(text: string, key: CryptoKey, iv: string) {
  return new TextDecoder().decode(
    await crypto.subtle.decrypt(
      {
        name: 'AES-GCM',
        iv: b642ab(iv),
      },
      key,
      b642ab(text),
    ),
  );
}
