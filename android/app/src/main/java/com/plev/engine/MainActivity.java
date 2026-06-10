package com.plev.engine;

import com.google.androidgamesdk.GameActivity;
import android.os.Bundle;

public class MainActivity extends GameActivity {
    static {
        System.loadLibrary("plev");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
    }
}
