package com.samuelkurtzer.ausfin

import android.os.Bundle
import android.os.SystemClock
import android.widget.Toast
import androidx.activity.OnBackPressedCallback
import androidx.activity.enableEdgeToEdge
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat

class MainActivity : TauriActivity() {
  override fun onCreate(savedInstanceState: Bundle?) {
    enableEdgeToEdge()
    super.onCreate(savedInstanceState)
    // Keep the webview out from under the status/navigation bars; the
    // WebView never sees CSS safe-area insets on Android, so pad natively.
    val root = findViewById<android.view.View>(android.R.id.content)
    root.setBackgroundColor(0xFF0B0B09.toInt())
    ViewCompat.setOnApplyWindowInsetsListener(root) { view, insets ->
      val bars = insets.getInsets(
        WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
      )
      view.setPadding(bars.left, bars.top, bars.right, bars.bottom)
      WindowInsetsCompat.CONSUMED
    }

    // A stray back gesture would otherwise quit and lose the user's place;
    // require a second press within two seconds.
    var lastBackPressMs = 0L
    onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
      override fun handleOnBackPressed() {
        val now = SystemClock.elapsedRealtime()
        if (now - lastBackPressMs < 2000L) {
          finish()
        } else {
          lastBackPressMs = now
          Toast.makeText(this@MainActivity, "Press back again to exit", Toast.LENGTH_SHORT).show()
        }
      }
    })
  }
}
